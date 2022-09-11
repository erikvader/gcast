use tokio_util::sync::CancellationToken;

use crate::{util::FutureCancel, Receiver, Sender};

pub async fn caster_actor(
    to_conn: Sender,
    mut from_conn: Receiver,
    canceltoken: CancellationToken,
) -> anyhow::Result<()> {
    loop {
        match from_conn.recv().cancellable(&canceltoken).await {
            None => {
                log::debug!("caster got cancelled");
                break;
            }
            Some(None) => {
                break;
            }
            Some(Some(msg)) => {
                log::debug!("got msg {:?}", msg);
                log::debug!("sending msg {:?}", msg);
                if to_conn.send(msg).await.is_err() {
                    log::warn!("connections seems to be down");
                }
            }
        }
    }

    canceltoken.cancelled().await;

    Ok(())
}
