use tokio_util::sync::CancellationToken;

use crate::{Receiver, Sender};

pub async fn caster_actor(
    mut to_conn: Sender,
    mut from_conn: Receiver,
    canceltoken: CancellationToken,
) {
    canceltoken.cancelled().await;
}
