use protocol::{
    to_client::front::Front,
    to_server::{mpvstart, playurlstart, ToServer},
};

use super::{Control, Jump, MachineResult, StateLogger};

pub(super) async fn play_url_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("PlayUrl");

    while let Some(msg) = ctrl.send_recv(Front::PlayUrl).await {
        match msg {
            ToServer::MpvStart(mpvstart::Url(mpvstart::url::Url { url, paused })) => {
                return Jump::mpv_url(url, paused)
            }
            ToServer::PlayUrlStart(playurlstart::Stop) => break,
            _ => logger.invalid_message(&msg),
        }
    }

    Ok(())
}
