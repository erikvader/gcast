mod frontjob;

use protocol::{
    to_server::{spotifystart, ToServer},
    Message,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{Receiver, Sender};

use self::frontjob::FrontJob;

async fn handle_msg(msg: Message, front: &mut FrontJob, to_conn: &Sender) {
    log::debug!("Handling message: {:?}", msg);
    assert!(msg.is_to_server());

    use ToServer::*;
    match msg.take_to_server() {
        SendStatus(_) => {
            if to_conn.send(front.status()).await.is_err() {
                log::warn!("seems like connections is down");
            }
        }
        MpvControl(_) => todo!(),
        MpvStart(_) => todo!(),
        SpotifyStart(spotifystart::Start) => {
            if front.is_spotify() {
                log::warn!("spotify is already running, ignoring start request");
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

pub async fn caster_actor(
    to_conn: Sender,
    mut from_conn: Receiver,
    canceltoken: CancellationToken,
) -> anyhow::Result<()> {
    let mut front = FrontJob::default();

    loop {
        select! {
            Some(msg) = from_conn.recv() => handle_msg(msg, &mut front, &to_conn).await,
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
