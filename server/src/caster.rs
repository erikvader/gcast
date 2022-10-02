mod frontjob;

use protocol::{
    to_server::{mpvstart, spotifystart, ToServer},
    Message,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{Receiver, Sender};

use self::frontjob::FrontJob;

async fn handle_msg(msg: Message, front: &mut FrontJob) {
    log::debug!("Handling message: {:?}", msg);
    assert!(msg.is_to_server());

    use ToServer::*;
    match msg.take_to_server() {
        SendStatus(_) => front.send_status().await,
        MpvControl(_) => todo!(),
        MpvStart(mpvstart::File(path)) => {
            if let Some(string) = path.to_str() {
                try_start_mpv(front, string.to_string());
            } else {
                log::error!("the path '{:?}' is not a valid UTF-8 string", path);
            }
        }
        MpvStart(mpvstart::Url(url)) => try_start_mpv(front, url),
        MpvStart(mpvstart::Stop) => {
            if !front.is_mpv() {
                log::warn!("mpv is not running, ignoring stop request");
            } else {
                log::info!("Killing mpv");
                front.kill().await;
            }
        }
        SpotifyStart(spotifystart::Start) => {
            if front.is_something() {
                log::warn!(
                    "'{}' is already running, ignoring start request",
                    front.name()
                );
            } else {
                log::info!("Starting spotify");
                front.start_spotify();
            }
        }
        SpotifyStart(spotifystart::Stop) => {
            if !front.is_spotify() {
                log::warn!("spotify is not running, ignoring stop request");
            } else {
                log::info!("Killing spotify");
                front.kill().await;
            }
        }
    }
}

fn try_start_mpv(front: &mut FrontJob, path: String) {
    if front.is_something() {
        log::warn!(
            "'{}' is already running, ignoring file request",
            front.name()
        );
    } else {
        log::info!("Starting mpv");
        front.start_mpv(path);
    }
}

pub async fn caster_actor(
    to_conn: Sender,
    mut from_conn: Receiver,
    canceltoken: CancellationToken,
) -> anyhow::Result<()> {
    let mut front = FrontJob::new(to_conn);

    loop {
        select! {
            Some(msg) = from_conn.recv() => handle_msg(msg, &mut front).await,
            _ = canceltoken.cancelled() => {
                log::debug!("caster got cancelled");
                break;
            },
            _ = front.wait() => {
                log::warn!("Application '{}' exited", front.name());
                front.kill().await;
            }
        }
    }

    if front.is_something() {
        log::info!(
            "Trying to exit caster, but killing '{}' first",
            front.name()
        );
        front.kill().await;
    }

    log::info!("Caster actor exited");
    Ok(())
}
