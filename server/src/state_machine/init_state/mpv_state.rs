use anyhow::Context;
use protocol::{
    to_client::front,
    to_server::{mpvstart, ToServer},
};
use tokio::select;

use crate::{
    filer::{cache_file, read_cache},
    mpv::{self},
};

use super::{Control, Jump, MachineResult, StateLogger};

pub(super) async fn mpv_url_state(
    ctrl: &mut Control,
    url: String, // TODO: use an URL type
    paused: bool,
) -> MachineResult<()> {
    let logger = StateLogger::new("MpvUrl");
    logger.info(format!("Playing URL: url={url}, paused={paused}"));
    mpv_state(ctrl, url, paused).await
}

pub(super) async fn mpv_file_state(
    ctrl: &mut Control,
    root: usize,
    path: String,
) -> MachineResult<()> {
    let logger = StateLogger::new("MpvFile");
    logger.info(format!("Playing file: root={root}, path={path}"));

    // TODO: the whole cache is re-read over and over, so cache the cache somehow? Maybe
    // save the roots in another file?
    let cache = read_cache(&cache_file())
        .await
        .context("failed to read the cache")?;
    let roots = cache.roots_path();

    match roots.get(root) {
        None => {
            logger.error(format!("Root {} out of range of 0..{}", root, roots.len()));
            Jump::user_error("Could not find file to play", "Root dir is out of range")
        }
        Some(r) => {
            assert!(path.starts_with('/'));
            assert!(!r.ends_with('/'));
            mpv_state(ctrl, r.to_string() + &path, false).await
        }
    }
}

async fn mpv_state(ctrl: &mut Control, path: String, paused: bool) -> MachineResult<()> {
    let logger = StateLogger::new("Mpv");
    logger.debug(format!("path={path}, paused={paused}"));

    ctrl.send(front::mpv::Load).await;

    let mut handle = mpv::mpv(&path, paused).context("creating mpv handle")?;

    let retval: MachineResult<()> = loop {
        select! {
            msg = ctrl.recv() => {
                match msg {
                    Some(ToServer::MpvStart(mpvstart::Stop)) | None => {
                        break Ok(())
                    },
                    Some(ToServer::MpvControl(mpvctrl)) => break_err!{
                        handle.command(mpvctrl.clone())
                            .with_context(|| format!("calling command {:?}", mpvctrl))
                    },
                    Some(m) => {
                        logger.invalid_message(&m);
                    },
                }
            }
            state = handle.next() => {
                match state {
                    Some(Ok(newstate)) => {
                        ctrl.send(newstate).await;
                    },
                    None => break Ok(()),
                    Some(Err(e)) => break Jump::user_error("Mpv play", e),
                }
            }
        }
    };

    logger.waiting("mpv handle to exit");
    let reason = handle.wait_until_closed().await;
    logger.debug(format!("exit reason: {reason:?}"));
    retval
}
