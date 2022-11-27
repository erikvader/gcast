use std::convert::Infallible;

use protocol::ToMessage;

use crate::{util::send_to_conn, Sender};

pub fn static_job_start(
    to_conn: Sender,
    front: impl ToMessage + Clone + Send + Sync + 'static,
) -> super::Job<(), Infallible> {
    super::Job::start(|mut rx| async move {
        send_to_conn(&to_conn, front.clone()).await;
        while let Some(jm) = rx.recv().await {
            assert!(
                jm.is_send_status(),
                "the received message must be a SendStatus"
            );
            send_to_conn(&to_conn, front.clone()).await;
        }
        Ok(())
    })
}
