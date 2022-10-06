use async_trait::async_trait;
use std::future::Future;
use tokio::{select, task::JoinHandle};
use tokio_util::sync::CancellationToken;

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

pub async fn join_handle_wait<T>(handle: &mut JoinHandle<T>) -> T {
    match handle.await {
        Err(err) if err.is_panic() => std::panic::resume_unwind(err.into_panic()),
        Err(err) if err.is_cancelled() => {
            panic!("Currently not supporting JoinHandle::abort")
        }
        Err(_) => unreachable!("A new variant of JoinError has been introduced"),
        Ok(x) => x,
    }
}

pub async fn join_handle_wait_take<T>(mut handle: JoinHandle<T>) -> T {
    join_handle_wait(&mut handle).await
}
