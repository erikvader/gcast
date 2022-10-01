use std::path::PathBuf;

use protocol::{
    to_client::{self, front},
    to_server::{mpvcontrol::MpvControl, mpvstart},
    Message, ToMessage,
};
use tokio::{select, sync::mpsc};

use crate::{
    job::{Job, JobMpsc, JobMsg},
    mpv,
    process::Process,
    Sender,
};

pub enum FrontJob {
    Spotify(JobMpsc<()>),
    Mpv(JobMpsc<MpvControl>),
    // FileSearch(JobOnce<JobMsg<FSControl>>),
    None(JobMpsc<()>),
}

impl FrontJob {
    pub async fn wait(&mut self) {
        use FrontJob::*;
        match self {
            Spotify(j) => j.wait().await,
            Mpv(j) => j.wait().await,
            None(j) => j.wait().await,
        }
    }

    pub fn name(&self) -> &'static str {
        use FrontJob::*;
        match self {
            Spotify(_) => "spotify",
            Mpv(_) => "mpv",
            None(_) => "nothing",
        }
    }

    pub async fn kill(&mut self, to_conn: Sender) {
        use FrontJob::*;
        match std::mem::replace(self, Self::none_job(to_conn)) {
            Spotify(j) => j.terminate_wait().await,
            Mpv(j) => j.terminate_wait().await,
            None(j) => j.terminate_wait().await,
        }
    }

    pub async fn send_status(&self) {
        use FrontJob::*;
        let res = match self {
            None(j) => j.send_status().await,
            Spotify(j) => j.send_status().await,
            Mpv(j) => j.send_status().await,
        };
        if res.is_err() {
            log::error!("couldn't request for status, job is down");
        }
    }

    pub fn is_something(&self) -> bool {
        use FrontJob::*;
        match self {
            None(_) => false,
            _ => true,
        }
    }

    pub fn is_spotify(&self) -> bool {
        use FrontJob::*;
        match self {
            Spotify(_) => true,
            _ => false,
        }
    }

    pub fn is_mpv(&self) -> bool {
        use FrontJob::*;
        match self {
            Mpv(_) => true,
            _ => false,
        }
    }

    pub fn none_job(to_conn: Sender) -> Self {
        Self::None(JobMpsc::start(|mut rx| async move {
            send_to_conn(&to_conn, front::None).await;
            while let Some(_) = rx.recv().await {
                send_to_conn(&to_conn, front::None).await;
            }
        }))
    }

    pub fn start_spotify(&mut self, to_conn: Sender) {
        *self = Self::Spotify(JobMpsc::start(|rx| async move {
            if let Err(e) = spotify(rx, to_conn).await {
                log::error!("Starting spotify failed with: {:?}", e);
            }
        }));
    }

    pub fn start_mpv(&mut self, file: String, to_conn: Sender) {
        *self = Self::Mpv(JobMpsc::start(|rx| async move {
            if let Err(e) = mpv(rx, file, to_conn).await {
                log::error!("Starting mpv failed with: {:?}", e);
            }
        }));
    }
}

async fn spotify(
    mut rx: mpsc::Receiver<JobMsg<()>>,
    to_conn: Sender,
) -> anyhow::Result<()> {
    send_to_conn(&to_conn, front::Spotify).await;
    let mut proc = Process::start("spotify")?;
    select! {
        _ = rx.recv() => {
            log::debug!("signal to terminate spotify received");
            proc.kill();
            let status = proc.wait().await?;
            log::debug!("spotify process exited with: {}", status);
            Ok(())
        },
        res = proc.wait() => {
            log::warn!("spotify process exited early with: {}", res?);
            Ok(())
        },
    }
}

async fn mpv(
    mut rx: mpsc::Receiver<JobMsg<MpvControl>>,
    file: String,
    to_conn: Sender,
) -> anyhow::Result<()> {
    send_to_conn(&to_conn, front::mpv::Load).await;
    let (handle, mut states) = mpv::mpv(&file)?;
    loop {
        select! {
            _ = rx.recv() => {},
            _ = states.next() => (),
        }
        break;
    }
    Ok(())
}

async fn send_to_conn(to_conn: &Sender, msg: impl ToMessage) {
    if to_conn.send(msg.to_message()).await.is_err() {
        log::warn!("seems like connections is down");
    }
}
