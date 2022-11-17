mod filerjob;
mod frontjob;
mod mpvjob;

use protocol::{
    to_server::{errormsgctrl, fsstart, mpvstart, spotifystart, ToServer},
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

        MpvStart(mpvstart::File(file)) => front.start_mpv_file(file).await,

        MpvStart(mpvstart::Url(s)) => front.start_mpv_url(s).await,
        MpvStart(mpvstart::Stop) => front.stop_mpv().await,
        MpvControl(ctrl) => front.send_mpv_ctrl(ctrl).await,

        SpotifyStart(spotifystart::Start) => front.start_spotify().await,
        SpotifyStart(spotifystart::Stop) => front.stop_spotify().await,

        FsStart(fsstart::Start) => front.start_filer().await,
        FsStart(fsstart::Stop) => front.stop_filer().await,
        FsControl(ctrl) => front.send_filer_ctrl(ctrl).await,

        ErrorMsgCtrl(errormsgctrl::Close) => front.close_error_message().await,
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
            res = front.wait() => {
                log::info!("Application '{}' exited", front.name());
                match res {
                    Ok(()) => front.kill().await,
                    Err(e) => {
                        log::error!("'{}' exited with an error: {:?}", front.name(), e);
                        front.error_message_err(format!("{} exited with an error", front.name()), &e).await;
                    }
                }
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
