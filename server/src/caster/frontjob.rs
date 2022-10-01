use std::io;

use protocol::{
    to_client::{self, front},
    to_server::mpvcontrol::MpvControl,
    Message, ToMessage,
};
use tokio::{select, sync::mpsc};

use crate::{job::JobMpsc, process::Process};

pub enum JobMsg<T> {
    SendStatus,
    Ctrl(T),
}

pub enum FrontJob {
    Spotify(JobMpsc<()>),
    Mpv(JobMpsc<JobMsg<MpvControl>>),
    // FileSearch(JobOnce<JobMsg<FSControl>>),
    None,
}

impl Default for FrontJob {
    fn default() -> Self {
        Self::None
    }
}

impl FrontJob {
    pub async fn wait(&mut self) {
        use FrontJob::*;
        match self {
            Spotify(j) => j.wait().await,
            Mpv(j) => j.wait().await,
            None => std::future::pending().await,
        }
    }

    pub fn name(&self) -> &'static str {
        use FrontJob::*;
        match self {
            Spotify(_) => "spotify",
            Mpv(_) => "mpv",
            None => "",
        }
    }

    pub async fn kill(&mut self) {
        use FrontJob::*;
        match std::mem::take(self) {
            Spotify(j) => j.terminate_wait().await,
            Mpv(j) => j.terminate_wait().await,
            None => (),
        }
    }

    pub fn status(&self) -> Message {
        use FrontJob::*;
        match self {
            None => front::None.to_message(),
            Spotify(_) => front::Spotify.to_message(),
            Mpv(j) => todo!(),
        }
    }

    pub fn is_something(&self) -> bool {
        use FrontJob::*;
        match self {
            None => false,
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

    pub fn start_spotify(&mut self) {
        *self = Self::Spotify(JobMpsc::start(|rx| async move {
            if let Err(e) = spotify(rx).await {
                log::error!("Starting spotify failed with: {:?}", e);
            }
        }));
    }
}

async fn spotify(mut rx: mpsc::Receiver<()>) -> anyhow::Result<()> {
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
