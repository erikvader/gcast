mod frontjob;

use protocol::{
    to_server::{fsstart, mpvstart, spotifystart, ToServer},
    Message,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{Receiver, Sender};

use self::frontjob::FrontJob;

async fn handle_msg(msg: Message, front: &mut FrontJob) {
    log::info!("Handling message: {:?}", msg);
    assert!(msg.is_to_server());

    use ToServer::*;
    match msg.take_to_server() {
        SendStatus(_) => front.send_status().await,
        MpvControl(ctrl) => front.send_mpv_ctrl(ctrl).await,
        FsControl(ctrl) => front.send_filer_ctrl(ctrl).await,
        MpvStart(mpvstart::File(s)) | MpvStart(mpvstart::Url(s)) => {
            try_start_mpv(front, s)
        }
        MpvStart(mpvstart::Stop) => {
            if !front.is_mpv() {
                log::warn!("Mpv is not running, ignoring stop request");
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
                log::warn!("Spotify is not running, ignoring stop request");
            } else {
                log::info!("Killing spotify");
                front.kill().await;
            }
        }
        FsStart(fsstart::Start) => {
            if front.is_something() {
                log::warn!(
                    "'{}' is already running, ignoring start request",
                    front.name()
                );
            } else {
                log::info!("Starting filer");
                front.start_filer();
            }
        }
        FsStart(fsstart::Stop) => {
            if !front.is_filer() {
                log::warn!("Filer is not running, ignoring stop request");
            } else {
                log::info!("Killing filer");
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
                log::debug!("Caster got cancelled");
                break;
            },
            _ = front.wait() => {
                log::info!("Application '{}' exited", front.name());
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
