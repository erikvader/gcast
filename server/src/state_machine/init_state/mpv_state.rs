use anyhow::Context;
use protocol::{
    to_client::front,
    to_server::{mpvstart, ToServer},
};
use tokio::select;

use crate::mpv::{self, MpvError};

use super::{Control, Jump, MachineResult, StateLogger};

pub(super) async fn mpv_url_state(ctrl: &mut Control, url: String) -> MachineResult<()> {
    let _logger = StateLogger::new("MpvUrl");
    mpv_state(ctrl, url).await
}

pub(super) async fn mpv_file_state(
    ctrl: &mut Control,
    root: usize,
    path: String,
) -> MachineResult<()> {
    let logger = StateLogger::new("MpvFile");
    let roots = crate::config::root_dirs();

    match roots.get(root) {
        None => {
            logger.error(format!("Root {} out of range of 0..{}", root, roots.len()));
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

    ctrl.send(front::mpv::Load).await;

    let mut handle = mpv::mpv(&path).context("creating mpv handle")?;

    let retval: MachineResult<()> = loop {
        select! {
            msg = ctrl.recv() => {
                match msg {
                    Some(ToServer::MpvStart(mpvstart::Stop)) | None => {
                        logger.attempt_exit();
                        break handle.quit().await.map_err(|e| e.into())
                    },
                    Some(ToServer::MpvControl(mpvctrl)) => break_err!(handle.command(&mpvctrl).await),
                    Some(m) => logger.invalid_message(&m),
                }
            }
            state = handle.next() => {
                match state.map(|s| s.to_client_state()) {
                    Ok(Some(newstate)) => ctrl.send(newstate).await,
                    Ok(None) => (),
                    Err(MpvError::Exited) => break Ok(()),
                    Err(e) => break Jump::user_error("Mpv play", e),
                }
            }
        }
    };

    logger.waiting("mpv handle to exit");
    handle.wait_until_closed().await;
    retval
}
