use std::convert::Infallible;

use protocol::{
    to_client::front::{self, errormsg},
    to_server::{
        fscontrol::FsControl, mpvcontrol::MpvControl, mpvstart, powerctrl::PowerCtrl,
        spotifyctrl::SpotifyCtrl,
    },
};

use crate::{
    filer::{self, FilerError},
    job::{handlejob::handle_job_start, staticjob::static_job_start, Job},
    mpv::{self, MpvError},
    process::Process,
    Sender,
};

use super::processjob::ProcessHandleJobError;

pub struct FrontJob {
    to_conn: Sender,
    var: Variant,
}

enum Variant {
    Spotify(Job<(), ProcessHandleJobError>),
    Mpv(Job<MpvControl, MpvError>),
    Filer(Job<FsControl, FilerError>),
    PlayUrl(Job<(), Infallible>),
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
            PlayUrl(j) => j.terminate(),
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
            PlayUrl(j) => j.wait().await?,
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

    pub async fn error_message_err<E, H>(&mut self, header: H, error: E)
    where
        E: std::fmt::Debug,
        H: ToString,
    {
        let header = header.to_string();
        let body = format!("{:?}", error);
        log::info!(
            "Showing error message '{}', from an error: '{:?}'",
            header,
            error
        );
        self.transition(|to_conn| Variant::error_job(to_conn, header, body))
            .await;
    }

    pub async fn error_message_str<H, B>(&mut self, header: H, body: B)
    where
        H: ToString,
        B: ToString,
    {
        let header = header.to_string();
        let body = body.to_string();
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
            PlayUrl(j) => j.send_status().await,
            ErrorMsg(j) => j.send_status().await,
            Filer(j) => j.send_status().await,
        };
        if res.is_err() {
            log::error!("Couldn't request for status, job is down");
        }
    }

    pub async fn powerctrl(&mut self, ctrl: PowerCtrl) {
        send_ctrl_check!(self, "nothing", self.is_none());
        match ctrl {
            PowerCtrl::Poweroff => {
                self.oneshot_process(crate::config::poweroff_exe().to_string())
                    .await
            }
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

    pub async fn send_spotify_ctrl(&mut self, ctrl: SpotifyCtrl) {
        send_ctrl_check!(self, "spotify", self.is_spotify());
        match ctrl {
            SpotifyCtrl::Fullscreen => {
                self.oneshot_process(crate::config::spotify_fullscreen_exe().to_string())
                    .await;
            }
        }
    }

    async fn oneshot_process(&mut self, exe: String) {
        let errmsg = format!("Failed to run process '{}'", exe);
        match Process::start(exe) {
            Err(e) => self.error_message_err(errmsg, e).await,
            Ok(mut p) => match p.wait().await.unwrap() {
                Err(e) => self.error_message_err(errmsg, e).await,
                Ok(exitstatus) => {
                    if !exitstatus.success() {
                        self.error_message_str(errmsg, exitstatus).await;
                    }
                }
            },
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

    pub fn is_play_url(&self) -> bool {
        matches!(self.var, Variant::PlayUrl(_))
    }

    pub fn is_error_message(&self) -> bool {
        matches!(self.var, Variant::ErrorMsg(_))
    }

    pub async fn start_spotify(&mut self) {
        start_check!(self, self.is_none());
        log::info!("Starting spotify"); // TODO: same name as variant::name()
        self.transition(Variant::spotify_job).await;
    }

    pub async fn stop_spotify(&mut self) {
        stop_check!("Spotify", self.is_spotify());
        self.kill().await;
    }

    pub async fn start_play_url(&mut self) {
        start_check!(self, self.is_none());
        log::info!("Starting play url");
        self.transition(Variant::play_url_job).await;
    }

    pub async fn stop_play_url(&mut self) {
        stop_check!("play url", self.is_play_url());
        self.kill().await;
    }

    pub async fn start_mpv_url(&mut self, url: String) {
        start_check!(self, self.is_none() || self.is_play_url());
        log::info!("Starting mpv with url");
        if !url.starts_with("http") {
            self.error_message_str("Not a valid URL", "").await;
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
                    "Could not find file to play",
                    "Root dir is out of range, try to refresh the cache",
                )
                .await;
            }
            Some(r) => {
                assert!(file.path.starts_with('/'));
                assert!(!r.ends_with('/'));
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
        Self::None(static_job_start(to_conn, front::None))
    }

    fn play_url_job(to_conn: Sender) -> Self {
        Self::PlayUrl(static_job_start(to_conn, front::PlayUrl))
    }

    fn error_job(to_conn: Sender, header: String, body: String) -> Self {
        let state = errormsg::ErrorMsg { header, body };
        Self::ErrorMsg(static_job_start(to_conn, state))
    }

    fn mpv_job(to_conn: Sender, file: String) -> Self {
        Self::Mpv(handle_job_start(to_conn, move || mpv::mpv(&file)))
    }

    fn filer_job(to_conn: Sender) -> Self {
        // Self::Filer(handle_job_start(to_conn, filer::filer))
        todo!()
    }

    fn spotify_job(to_conn: Sender) -> Self {
        Self::Spotify(handle_job_start(to_conn, move || {
            Process::start(crate::config::spotify_exe().to_string())
        }))
    }

    pub fn name(&self) -> &'static str {
        use Variant::*;
        match self {
            Spotify(_) => "spotify",
            Mpv(_) => "mpv",
            PlayUrl(_) => "play url",
            Filer(_) => "filesearch",
            ErrorMsg(_) => "error message",
            None(_) => "nothing",
        }
    }
}
