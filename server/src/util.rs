use async_trait::async_trait;
use std::future::Future;
use tokio::select;
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
