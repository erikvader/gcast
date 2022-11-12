use std::{convert::Infallible, io};

use protocol::{
    to_client::front::{self, errormsg},
    to_server::{fscontrol::FsControl, mpvcontrol::MpvControl, mpvstart},
    ToMessage,
};
use tokio::{select, sync::mpsc};

use crate::{
    filer::{self, FilerError, FilerResult},
    job::{Job, JobMsg},
    mpv::{self, MpvError, MpvResult},
    process::Process,
    Sender,
};

pub struct FrontJob {
    to_conn: Sender,
    var: Variant,
}

enum Variant {
    Spotify(Job<(), io::Error>),
    Mpv(Job<MpvControl, MpvError>),
    Filer(Job<FsControl, FilerError>),
    ErrorMsg(Job<(), Infallible>),
    None(Job<(), Infallible>),
}

macro_rules! start_check {
    ($self:ident, $continueif:expr) => {
        if !$continueif {
            log::warn!(
                "'{}' is already running, ignoring start request",
                $self.name()
            );
            return;
        }
    };
}

macro_rules! stop_check {
    ($me:expr, $continueif:expr) => {
        if !$continueif {
            log::warn!("{} is not running, ignoring stop request", $me);
            return;
        }
    };
}

macro_rules! send_ctrl_check {
    ($self:ident, $me:expr, $continueif:expr) => {
        if !$continueif {
            log::warn!(
                "Trying to send ctrl message, but {} is not active, '{}' is",
                $me,
                $self.name()
            );
            return;
        }
    };
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
            Spotify(j) => j.terminate(),
            Mpv(j) => j.terminate(),
            Filer(j) => j.terminate(),
            ErrorMsg(j) => j.terminate(),
            None(j) => j.terminate(),
        }
        if let Err(e) = self.wait().await {
            log::warn!("Error while terminating job in transition: {:?}", e);
        }
        self.var = next_state(self.to_conn.clone());
        log::debug!("Transitioned into {}", self.name());
    }

    pub async fn wait(&mut self) -> anyhow::Result<()> {
        use Variant::*;
        match &mut self.var {
            Spotify(j) => j.wait().await?,
            Mpv(j) => j.wait().await?,
            Filer(j) => j.wait().await?,
            ErrorMsg(j) => j.wait().await?,
            None(j) => j.wait().await?,
        }
        Ok(())
    }

    pub fn name(&self) -> &'static str {
        self.var.name()
    }

    pub async fn kill(&mut self) {
        log::info!("Killing {}", self.name());
        self.transition(Variant::none_job).await;
    }

    pub async fn error_message_err<E>(&mut self, header: String, error: &E)
    where
        E: std::fmt::Debug,
    {
        log::info!(
            "Showing error message '{}', from an error: '{:?}'",
            header,
            error
        );
        let body = format!("{:?}", error);
        self.transition(|to_conn| Variant::error_job(to_conn, header, body))
            .await;
    }

    pub async fn error_message_str(&mut self, header: String, body: String) {
        log::info!(
            "Showing error message '{}', with a message: '{}'",
            header,
            body
        );
        self.transition(|to_conn| Variant::error_job(to_conn, header, body))
            .await;
    }

    pub async fn send_status(&self) {
        use Variant::*;
        let res = match &self.var {
            None(j) => j.send_status().await,
            Spotify(j) => j.send_status().await,
            Mpv(j) => j.send_status().await,
            ErrorMsg(j) => j.send_status().await,
            Filer(j) => j.send_status().await,
        };
        if res.is_err() {
            log::error!("Couldn't request for status, job is down");
        }
    }

    pub async fn send_mpv_ctrl(&self, ctrl: MpvControl) {
        send_ctrl_check!(self, "mpv", self.is_mpv());
        match &self.var {
            Variant::Mpv(j) => {
                if j.send_ctrl(ctrl).await.is_err() {
                    log::error!("Couldn't send ctrl, job is down");
                }
            }
            _ => unreachable!("must be mpv due to check"),
        }
    }

    pub async fn send_filer_ctrl(&self, ctrl: FsControl) {
        send_ctrl_check!(self, "filer", self.is_filer());
        match &self.var {
            Variant::Filer(j) => {
                if j.send_ctrl(ctrl).await.is_err() {
                    log::error!("Couldn't send ctrl, job is down");
                }
            }
            _ => unreachable!("must be filer due to check"),
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

    pub fn is_error_message(&self) -> bool {
        matches!(self.var, Variant::ErrorMsg(_))
    }

    pub async fn start_spotify(&mut self) {
        start_check!(self, self.is_none());
        log::info!("Starting spotify");
        self.transition(Variant::spotify_job).await;
    }

    pub async fn stop_spotify(&mut self) {
        stop_check!("Spotify", self.is_spotify());
        self.kill().await;
    }

    pub async fn start_mpv_url(&mut self, url: String) {
        start_check!(self, self.is_none());
        log::info!("Starting mpv with url");
        if !url.starts_with("http") {
            self.error_message_str("Not a valid URL".to_string(), "".to_string())
                .await;
        } else {
            self.transition(|to_conn| Variant::mpv_job(to_conn, url))
                .await;
        }
    }

    pub async fn start_mpv_file(&mut self, file: mpvstart::File) {
        start_check!(self, self.is_none() || self.is_filer());
        log::info!("Starting mpv with file: {:?}", file);

        let roots = crate::config::root_dirs();
        match roots.get(file.root) {
            None => {
                log::error!("Root {} out of range of 0..{}", file.root, roots.len());
                self.error_message_str(
                    "Could not find file to play".to_string(),
                    "Root dir is out of range, try to refresh the cache".to_string(),
                )
                .await;
            }
            Some(r) => {
                assert!(file.path.starts_with("/"));
                assert!(!r.ends_with("/"));
                self.transition(|to_conn| {
                    Variant::mpv_job(to_conn, r.to_string() + &file.path)
                })
                .await
            }
        }
    }

    pub async fn stop_mpv(&mut self) {
        stop_check!("Mpv", self.is_mpv());
        self.kill().await;
    }

    pub async fn start_filer(&mut self) {
        start_check!(self, self.is_none());
        log::info!("Starting filer");
        self.transition(Variant::filer_job).await;
    }

    pub async fn stop_filer(&mut self) {
        stop_check!("Filer", self.is_filer());
        self.kill().await;
    }

    pub async fn close_error_message(&mut self) {
        stop_check!("Error message", self.is_error_message());
        self.kill().await;
    }
}

impl Variant {
    fn none_job(to_conn: Sender) -> Self {
        Self::None(Job::start(|mut rx| async move {
            send_to_conn(&to_conn, front::None).await;
            while let Some(jm) = rx.recv().await {
                assert!(
                    jm.is_send_status(),
                    "the received message must be a SendStatus"
                );
                send_to_conn(&to_conn, front::None).await;
            }
            Ok(())
        }))
    }

    fn error_job(to_conn: Sender, header: String, body: String) -> Self {
        Self::ErrorMsg(Job::start(|mut rx| async move {
            let state = errormsg::ErrorMsg { header, body };
            send_to_conn(&to_conn, state.clone()).await;
            while let Some(jm) = rx.recv().await {
                assert!(
                    jm.is_send_status(),
                    "the received message must be a SendStatus"
                );
                send_to_conn(&to_conn, state.clone()).await;
            }
            Ok(())
        }))
    }

    fn mpv_job(to_conn: Sender, file: String) -> Self {
        Variant::Mpv(Job::start(move |rx| mpv(rx, file, to_conn)))
    }

    fn filer_job(to_conn: Sender) -> Self {
        Variant::Filer(Job::start(move |rx| filer(rx, to_conn)))
    }

    fn spotify_job(to_conn: Sender) -> Self {
        Variant::Spotify(Job::start(move |rx| spotify(rx, to_conn)))
    }

    pub fn name(&self) -> &'static str {
        use Variant::*;
        match self {
            Spotify(_) => "spotify",
            Mpv(_) => "mpv",
            Filer(_) => "filesearch",
            ErrorMsg(_) => "error message",
            None(_) => "nothing",
        }
    }
}

async fn spotify(mut rx: mpsc::Receiver<JobMsg<()>>, to_conn: Sender) -> io::Result<()> {
    send_to_conn(&to_conn, front::Spotify).await;
    let mut proc = Process::start(crate::config::spotify_exe())?;

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
    file: String,
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
