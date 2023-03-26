use anyhow::Context;
use protocol::to_server::{fsstart, playurlstart, powerctrl, spotifystart};
use protocol::{
    to_client::front::Front,
    to_server::{mpvstart, ToServer},
};

use crate::process::Process;

use self::mpv_state::{mpv_file_state, mpv_url_state};
use self::play_url_state::play_url_state;
use self::spotify_state::spotify_state;

use super::*;

mod error_msg_state;
mod filer_state;
mod mpv_state;
mod play_url_state;
mod spotify_state;

pub(super) async fn init_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("Init");
    let mut queue = InjectableQueue::new();

    while let Some(msg) = queue.pop_or(|| ctrl.send_recv(Front::None)).await {
        let res: MachineResult<()> = match msg {
            ToServer::PowerCtrl(powerctrl::Poweroff) => {
                let name = crate::config::poweroff_exe();
                Process::oneshot(name.to_string())
                    .await
                    .context("running poweroff exe")
                    .jump_user_error("Failed to run poweroff exe")
                    .map(|exit| {
                        logger.process_done(name, exit);
                        ()
                    })
            }
            ToServer::MpvStart(mpvstart::Url(url)) => {
                mpv_url_state(ctrl, url).await.context("mpv url")
            }
            ToServer::MpvStart(mpvstart::File(file)) => {
                mpv_file_state(ctrl, file.root, file.path)
                    .await
                    .context("mpv file")
            }
            ToServer::SpotifyStart(spotifystart::Start) => {
                spotify_state(ctrl).await.context("spotify")
            }
            ToServer::FsStart(fsstart::Start) => {
                filer_state::filer_state(ctrl).await.context("filer")
            }
            ToServer::PlayUrlStart(playurlstart::Start) => {
                play_url_state(ctrl).await.context("play url")
            }
            _ => {
                logger.invalid_message(&msg);
                Ok(())
            }
        };

        if let Err(e) = res.context(format!("in state '{}'", logger.name())) {
            match e.downcast() {
                Ok(Jump::Mpv(mpvstart)) => queue.inject(mpvstart.into()),
                Ok(Jump::UserError { header, body }) => {
                    error_msg_state::error_msg_state(ctrl, header, body)
                        .await
                        .context("error message")?
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}
