use std::{io, path::PathBuf};

use protocol::{
    to_client::{self, front},
    to_server::{fscontrol::FsControl, mpvcontrol::MpvControl, mpvstart},
    Message, ToMessage,
};
use tokio::{join, select, sync::mpsc};

use crate::{
    filer::{self, FilerError, FilerResult},
    job::{Job, JobMpsc, JobMsg},
    mpv::{self, MpvError, MpvResult},
    process::Process,
    Sender,
};

pub struct FrontJob {
    to_conn: Sender,
    var: Variant,
}

enum Variant {
    Spotify(JobMpsc<()>),
    Mpv(JobMpsc<MpvControl>),
    Filer(JobMpsc<FsControl>),
    None(JobMpsc<()>),
}

impl FrontJob {
    pub fn new(to_conn: Sender) -> Self {
        Self {
            var: Variant::none_job(to_conn.clone()),
            to_conn,
        }
    }

    pub async fn wait(&mut self) {
        use Variant::*;
        match &mut self.var {
            Spotify(j) => j.wait().await,
            Mpv(j) => j.wait().await,
            Filer(j) => j.wait().await,
            None(j) => j.wait().await,
        }
    }

    pub fn name(&self) -> &'static str {
        use Variant::*;
        match self.var {
            Spotify(_) => "spotify",
            Mpv(_) => "mpv",
            Filer(_) => "filesearch",
            None(_) => "nothing",
        }
    }

    pub async fn kill(&mut self) {
        use Variant::*;
        match &mut self.var {
            Spotify(j) => j.terminate_wait().await,
            Mpv(j) => j.terminate_wait().await,
            Filer(j) => j.terminate_wait().await,
            None(j) => j.terminate_wait().await,
        }
        self.var = Variant::none_job(self.to_conn.clone());
    }

    pub async fn send_status(&self) {
        use Variant::*;
        let res = match &self.var {
            None(j) => j.send_status().await,
            Spotify(j) => j.send_status().await,
            Mpv(j) => j.send_status().await,
            Filer(j) => j.send_status().await,
        };
        if res.is_err() {
            log::error!("Couldn't request for status, job is down");
        }
    }

    pub async fn send_mpv_ctrl(&self, ctrl: MpvControl) {
        if let Variant::Mpv(j) = &self.var {
            if j.send_ctrl(ctrl).await.is_err() {
                log::error!("Couldn't send ctrl, job is down");
            }
        } else {
            log::warn!(
                "Trying to send '{:?}' but mpv is not active, '{}' is",
                ctrl,
                self.name()
            );
        }
    }

    pub fn is_something(&self) -> bool {
        !matches!(self.var, Variant::None(_))
    }

    pub fn is_spotify(&self) -> bool {
        matches!(self.var, Variant::Spotify(_))
    }

    pub fn is_mpv(&self) -> bool {
        matches!(self.var, Variant::Mpv(_))
    }

    pub fn is_filer(&self) -> bool {
        matches!(self.var, Variant::Filer(_))
    }

    pub fn start_spotify(&mut self) {
        let to_conn = self.to_conn.clone();
        self.var = Variant::Spotify(JobMpsc::start(|rx| async move {
            if let Err(e) = spotify(rx, to_conn).await {
                log::error!("Starting spotify failed with: {}", e);
            }
        }));
    }

    pub fn start_mpv(&mut self, file: String) {
        let to_conn = self.to_conn.clone();
        self.var = Variant::Mpv(JobMpsc::start(|rx| async move {
            if let Err(e) = mpv(rx, &file, to_conn).await {
                log::error!("Starting mpv failed with: {}", e);
            }
        }));
    }

    pub fn start_filer(&mut self) {
        let to_conn = self.to_conn.clone();
        self.var = Variant::Filer(JobMpsc::start(|rx| async move {
            if let Err(e) = filer(rx, to_conn).await {
                log::error!("Starting filer failed with: {}", e);
            }
        }));
    }
}

impl Variant {
    fn none_job(to_conn: Sender) -> Self {
        Self::None(JobMpsc::start(|mut rx| async move {
            send_to_conn(&to_conn, front::None).await;
            while let Some(_) = rx.recv().await {
                send_to_conn(&to_conn, front::None).await;
            }
        }))
    }
}

async fn spotify(mut rx: mpsc::Receiver<JobMsg<()>>, to_conn: Sender) -> io::Result<()> {
    send_to_conn(&to_conn, front::Spotify).await;
    let mut proc = Process::start("spotify")?;

    loop {
        select! {
            msg = rx.recv() => {
                match msg {
                    None => {
                        log::debug!("Signal to terminate spotify received");
                        proc.kill();
                        let status = proc.wait().await?;
                        log::debug!("Spotify process exited with: {}", status);
                        break Ok(());
                    }
                    Some(_) => send_to_conn(&to_conn, front::Spotify).await,
                }
            },
            res = proc.wait() => {
                log::warn!("Spotify process exited early with: {}", res?);
                break Ok(());
            },
        }
    }
}

async fn mpv(
    mut rx: mpsc::Receiver<JobMsg<MpvControl>>,
    file: &str,
    to_conn: Sender,
) -> MpvResult<()> {
    let mut handle = mpv::mpv(&file)?;

    let mut last_state = front::mpv::Load;
    let retval = loop {
        select! {
            msg = rx.recv() => {
                match msg {
                    None => {
                        log::debug!("Mpv exit signal received");
                        handle.quit().await.ok();
                        break Ok(());
                    },
                    Some(JobMsg::SendStatus) => send_to_conn(&to_conn, last_state.clone()).await,
                    Some(JobMsg::Ctrl(ctrl)) => if let Err(e) = handle.command(&ctrl).await {
                        break Err(e);
                    },
                }
            },
            state = handle.next() => {
                match state.map(|s| s.to_client_state()) {
                    Ok(Some(s)) => {
                        last_state = s.clone();
                        send_to_conn(&to_conn, s).await;
                    }
                    Ok(None) => (),
                    Err(MpvError::Exited) => break Ok(()),
                    Err(e) => {
                        break Err(e);
                    }
                }
            }
        }
    };

    handle.wait_until_closed().await;
    retval
}

async fn filer(
    mut rx: mpsc::Receiver<JobMsg<FsControl>>,
    to_conn: Sender,
) -> FilerResult<()> {
    let mut handle = filer::filer()?;

    let mut last_state = front::filesearch::FileSearch::default();
    loop {
        select! {
            msg = rx.recv() => {
                match msg {
                    None => {
                        log::debug!("Filer exit signal receiver");
                        // TODO: handle kill
                        // TODO: wait for filer to exit
                        break;
                    },
                    Some(JobMsg::SendStatus) => send_to_conn(&to_conn, last_state.clone()).await,
                    Some(JobMsg::Ctrl(FsControl::Search(s))) => handle.search(s).await?,
                    Some(JobMsg::Ctrl(FsControl::RefreshCache)) => handle.refresh_cache().await?,
                }
            },
            state = handle.next() => {
                match state {
                    Ok(s) => {
                        last_state = s.clone();
                        send_to_conn(&to_conn, s).await;
                    }
                    Err(FilerError::Exited) => break,
                    Err(e) => return Err(e),
                }
            }
        }
    }

    Ok(())
}

async fn send_to_conn(to_conn: &Sender, msg: impl ToMessage) {
    if to_conn.send(msg.to_message()).await.is_err() {
        log::warn!("Seems like connections is down");
    }
}
