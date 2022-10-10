use std::io;

use protocol::{
    to_client::front,
    to_server::{fscontrol::FsControl, mpvcontrol::MpvControl},
    ToMessage,
};
use tokio::{select, sync::mpsc};

use crate::{
    filer::{self, FilerError, FilerResult},
    job::{JobMpsc, JobMsg},
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
    // TODO: View that display all logs? Or send every log message to the client if one is connected?
}

impl FrontJob {
    pub fn new(to_conn: Sender) -> Self {
        Self {
            var: Variant::none_job(to_conn.clone()),
            to_conn,
        }
    }

    async fn transition<F>(&mut self, next_state: F)
    where
        F: FnOnce(Sender) -> Variant,
    {
        log::debug!("Transitioning, waiting for {} to terminate", self.name());
        use Variant::*;
        match &mut self.var {
            Spotify(j) => j.terminate_wait().await,
            Mpv(j) => j.terminate_wait().await,
            Filer(j) => j.terminate_wait().await,
            None(j) => j.terminate_wait().await,
        }
        self.var = next_state(self.to_conn.clone());
        log::debug!("Transitioned into {}", self.name());
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
        self.var.name()
    }

    pub async fn kill(&mut self) {
        log::info!("Killing {}", self.name());
        self.transition(Variant::none_job).await;
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

    pub async fn send_filer_ctrl(&self, ctrl: FsControl) {
        if let Variant::Filer(j) = &self.var {
            if j.send_ctrl(ctrl).await.is_err() {
                log::error!("Couldn't send ctrl, job is down");
            }
        } else {
            log::warn!(
                "Trying to send '{:?}' but filer is not active, '{}' is",
                ctrl,
                self.name()
            );
        }
    }

    pub fn is_something(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        matches!(self.var, Variant::None(_))
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

    pub async fn start_spotify(&mut self) {
        if self.is_something() {
            log::warn!(
                "'{}' is already running, ignoring spotify start request",
                self.name()
            );
            return;
        }

        log::info!("Starting spotify");
        self.transition(Variant::spotify_job).await;
    }

    pub async fn stop_spotify(&mut self) {
        if !self.is_spotify() {
            log::warn!("Spotify is not running, ignoring stop request");
            return;
        }
        self.kill().await;
    }

    pub async fn start_mpv_url(&mut self, url: String) {
        if self.is_something() {
            log::warn!(
                "'{}' is already running, ignoring mpv start request",
                self.name()
            );
            return;
        }

        log::info!("Starting mpv with url");
        self.transition(|to_conn| Variant::mpv_job(to_conn, url))
            .await;
    }

    pub async fn start_mpv_file(&mut self, file: String) {
        if self.is_something() && !self.is_filer() {
            log::warn!(
                "'{}' is already running, ignoring mpv start request",
                self.name()
            );
            return;
        }

        log::info!("Starting mpv with file");
        self.transition(|to_conn| Variant::mpv_job(to_conn, file))
            .await;
    }

    pub async fn stop_mpv(&mut self) {
        if !self.is_mpv() {
            log::warn!("Mpv is not running, ignoring stop request");
            return;
        }
        self.kill().await;
    }

    pub async fn start_filer(&mut self) {
        if self.is_something() {
            log::warn!(
                "'{}' is already running, ignoring filer start request",
                self.name()
            );
            return;
        }

        log::info!("Starting filer");
        self.transition(Variant::filer_job).await;
    }

    pub async fn stop_filer(&mut self) {
        if !self.is_filer() {
            log::warn!("Filer is not running, ignoring stop request");
            return;
        }
        self.kill().await;
    }
}

impl Variant {
    fn none_job(to_conn: Sender) -> Self {
        Self::None(JobMpsc::start(|mut rx| async move {
            send_to_conn(&to_conn, front::None).await;
            while let Some(jm) = rx.recv().await {
                assert!(jm.is_send_status(), "there is no way to send Ctrl(T) here");
                send_to_conn(&to_conn, front::None).await;
            }
        }))
    }

    fn mpv_job(to_conn: Sender, file: String) -> Self {
        Variant::Mpv(JobMpsc::start(|rx| async move {
            if let Err(e) = mpv(rx, &file, to_conn).await {
                log::error!("Starting mpv failed with: {}", e);
            }
        }))
    }

    fn filer_job(to_conn: Sender) -> Self {
        Variant::Filer(JobMpsc::start(|rx| async move {
            if let Err(e) = filer(rx, to_conn).await {
                log::error!("Starting filer failed with: {}", e);
            }
        }))
    }

    fn spotify_job(to_conn: Sender) -> Self {
        Variant::Spotify(JobMpsc::start(|rx| async move {
            if let Err(e) = spotify(rx, to_conn).await {
                log::error!("Starting spotify failed with: {}", e);
            }
        }))
    }

    pub fn name(&self) -> &'static str {
        use Variant::*;
        match self {
            Spotify(_) => "spotify",
            Mpv(_) => "mpv",
            Filer(_) => "filesearch",
            None(_) => "nothing",
        }
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
                    Some(jm) => {
                        assert!(jm.is_send_status(), "there is no way to send Ctrl(T) here");
                        send_to_conn(&to_conn, front::Spotify).await;
                    }
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
                    Some(JobMsg::Ctrl(ctrl)) => break_err!(handle.command(&ctrl).await),
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

    log::debug!("Waiting for mpv handle to exit");
    handle.wait_until_closed().await;
    retval
}

async fn filer(
    mut rx: mpsc::Receiver<JobMsg<FsControl>>,
    to_conn: Sender,
) -> FilerResult<()> {
    let mut handle = filer::filer()?;

    let mut last_state = front::filesearch::FileSearch::default();
    let retval = loop {
        select! {
            msg = rx.recv() => {
                match msg {
                    None => {
                        log::debug!("Filer exit signal receiver");
                        handle.kill();
                        break Ok(());
                    },
                    Some(JobMsg::SendStatus) => send_to_conn(&to_conn, last_state.clone()).await,
                    Some(JobMsg::Ctrl(FsControl::Search(s))) => break_err!(handle.search(s).await),
                    Some(JobMsg::Ctrl(FsControl::RefreshCache)) => break_err!(handle.refresh_cache().await),
                }
            },
            state = handle.next() => {
                match state {
                    Ok(s) => {
                        last_state = s.clone();
                        send_to_conn(&to_conn, s).await;
                    }
                    Err(FilerError::Exited) => break Ok(()),
                    Err(e) => break Err(e),
                }
            }
        }
    };

    log::debug!("Waiting for filer handle to exit");
    handle.wait_until_closed().await;
    retval
}

async fn send_to_conn(to_conn: &Sender, msg: impl ToMessage) {
    if to_conn.send(msg.to_message()).await.is_err() {
        log::warn!("Seems like connections is down");
    }
}
