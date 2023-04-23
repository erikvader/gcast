use std::time::Duration;

use futures_util::Sink;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message as TungMsg;

use crate::{repeatable_oneshot, util::join_handle_wait_take};

const RATE: Duration = Duration::from_micros(16_700);

// Rate limiter
pub struct Tractor<S> {
    handle: JoinHandle<anyhow::Result<S>>,
    sender: repeatable_oneshot::Sender<protocol::ToClient>,
}

impl<S> Tractor<S>
where
    S: Sink<TungMsg> + Unpin + Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(mut sink: S) -> Self {
        let (tx, rx) = repeatable_oneshot::repeat_oneshot();
        Self {
            handle: tokio::task::spawn(async move {
                let mut interval = tokio::time::interval(RATE);
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                let res: anyhow::Result<()> = loop {
                    let tosend: protocol::ToClient = match rx.recv().await {
                        Ok(it) => it,
                        Err(_) => break Ok(()),
                    };

                    break_err!(crate::connections::ws_send(tosend, &mut sink).await);
                    interval.tick().await;
                };

                res.and(Ok(sink))
            }),
            sender: tx,
        }
    }

    pub async fn send(
        &self,
        msg: protocol::ToClient,
    ) -> Result<(), repeatable_oneshot::OtherEndClosed> {
        self.sender.send(msg).await
    }

    pub async fn close(self) -> anyhow::Result<S> {
        drop(self.sender);
        join_handle_wait_take(self.handle).await
    }
}
