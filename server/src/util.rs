use async_trait::async_trait;
use protocol::ToMessage;
use std::future::Future;
use tokio::{select, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::Sender;

#[async_trait]
pub trait FutureCancel
where
    Self: Future,
{
    async fn cancellable(self, token: &CancellationToken) -> Option<Self::Output>;
}

#[async_trait]
impl<F> FutureCancel for F
where
    F: Future + Send,
{
    async fn cancellable(self, token: &CancellationToken) -> Option<Self::Output> {
        select! {
            _ = token.cancelled() => {None}
            x = self => {Some(x)}
        }
    }
}

pub async fn join_handle_wait_take<T>(mut handle: JoinHandle<T>) -> T {
    join_handle_wait(&mut handle).await
}

pub async fn join_handle_wait<T>(handle: &mut JoinHandle<T>) -> T {
    join_handle_unwrap(handle.await)
}

pub fn join_handle_unwrap<T>(awaited_handle: Result<T, tokio::task::JoinError>) -> T {
    match awaited_handle {
        Err(err) if err.is_panic() => std::panic::resume_unwind(err.into_panic()),
        Err(err) if err.is_cancelled() => {
            panic!("Currently not supporting JoinHandle::abort")
        }
        Err(_) => unreachable!("A new variant of JoinError has been introduced"),
        Ok(x) => x,
    }
}

macro_rules! break_err {
    ($e:expr) => {
        match $e {
            Ok(it) => it,
            Err(e) => break Err(e.into()),
        }
    };
}

pub async fn send_to_conn(to_conn: &Sender, msg: impl ToMessage) {
    if to_conn.send(msg.to_message()).await.is_err() {
        log::warn!("Seems like connections is down");
    }
}
