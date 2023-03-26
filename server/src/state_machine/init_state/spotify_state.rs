use anyhow::Context;
use protocol::{
    to_client::front::Front,
    to_server::{spotifyctrl, spotifystart, ToServer},
};
use tokio::select;

use crate::{process::Process, state_machine::JumpableError};

use super::{Control, MachineResult, StateLogger};

pub(super) async fn spotify_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("Spotify");

    ctrl.send(Front::Spotify).await;

    let mut proc = Process::start(crate::config::spotify_exe().to_string())
        .context("spawning spotify process")
        .jump_user_error("Failed to spawn spotify")?;

    let ret = loop {
        select! {
            msg = ctrl.recv() => {
                match msg {
                    Some(ToServer::SpotifyStart(spotifystart::Stop)) | None => {
                        logger.attempt_exit();
                        proc.kill();
                        logger.waiting("process to exit");
                        break proc.wait().await.expect("should not be awaited to completion more than once");
                    },
                    Some(ToServer::SpotifyCtrl(spotifyctrl::Fullscreen)) => {
                        let proc_name = crate::config::spotify_fullscreen_exe();
                        let proc = Process::oneshot(proc_name.to_string()).await
                            .context("spawning spotify fullscreen exe")
                            .jump_user_error("Failed to run spotify fullscreen exe")?;
                        logger.process_done(proc_name, proc);
                    },
                    Some(m) => logger.invalid_message(&m),
                }
            }
            ret = proc.wait() => {
                logger.warn("process exited on its own");
                break ret.expect("should not be awaited to completion more than once");
            }
        }
    };

    logger.process_done(proc.name(), ret?);

    Ok(())
}
