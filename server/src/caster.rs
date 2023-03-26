use tokio_util::sync::CancellationToken;

use crate::{state_machine, Receiver, Sender};

pub async fn caster_actor(
    to_conn: Sender,
    from_conn: Receiver,
    canceltoken: CancellationToken,
) -> anyhow::Result<()> {
    let ret = state_machine::state_start(from_conn, to_conn, canceltoken).await;
    log::info!("Caster actor exited");
    ret
}
