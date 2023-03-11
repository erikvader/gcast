use anyhow::Context;
use protocol::{
    to_client::front::Front,
    to_server::{
        mpvstart::{self, MpvStart},
        playurlstart, ToServer,
    },
};

use crate::mpv::{self, mpv};

use super::{Control, Jump, MachineResult, StateLogger};

pub(super) async fn mpv_url_state(ctrl: &mut Control, url: String) -> MachineResult<()> {
    mpv_state(ctrl, url).await
}

pub(super) async fn mpv_file_state(
    ctrl: &mut Control,
    root: usize,
    path: String,
) -> MachineResult<()> {
    let roots = crate::config::root_dirs();
    match roots.get(root) {
        None => {
            log::error!("Root {} out of range of 0..{}", root, roots.len());
            Jump::user_error(
                "Could not find file to play",
                "Root dir is out of range, try to refresh the cache",
            )
        }
        Some(r) => {
            assert!(path.starts_with('/'));
            assert!(!r.ends_with('/'));
            mpv_state(ctrl, r.to_string() + &path).await
        }
    }
}

async fn mpv_state(ctrl: &mut Control, path: String) -> MachineResult<()> {
    let logger = StateLogger::new("Mpv");
    let handle = mpv::mpv(&path).context("creating mpv handle")?;

    // TODO:
    while let Some(msg) = ctrl.send_recv(Front::PlayUrl).await {
        match msg {
            ToServer::MpvStart(mpvstart::Url(url)) => return Jump::mpv_url(url),
            ToServer::PlayUrlStart(playurlstart::Stop) => break,
            _ => logger.invalid_message(&msg),
        }
    }

    Ok(())
}
