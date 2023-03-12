use protocol::{
    to_client::front::Front,
    to_server::{mpvstart, playurlstart, ToServer},
};

use super::{Control, Jump, MachineResult, StateLogger};

pub(super) async fn play_url_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("PlayUrl");

    while let Some(msg) = ctrl.send_recv(Front::PlayUrl).await {
        match msg {
            // TODO: create its own message for this?
            ToServer::MpvStart(mpvstart::Url(url)) => return Jump::mpv_url(url),
            ToServer::PlayUrlStart(playurlstart::Stop) => break,
            _ => logger.invalid_message(&msg),
        }
    }

    Ok(())
}
