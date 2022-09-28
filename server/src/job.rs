use crate::repeatable_oneshot as RO;
use std::future::Future;
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

pub struct Job<T> {
    handle: JoinHandle<()>,
    tx: T,
}

pub type JobMpsc<M> = Job<mpsc::Sender<M>>;
pub type JobOne<M> = Job<RO::Sender<M>>;

impl<T> Job<T> {
    fn new(tx: T, handle: JoinHandle<()>) -> Self {
        Job { tx, handle }
    }

    pub async fn terminate_wait(self) {
        drop(self.tx);
        match self.handle.await {
            Err(err) if err.is_panic() => std::panic::resume_unwind(err.into_panic()),
            Err(_) => unreachable!("There is no way to call abort on the handle"),
            Ok(()) => (),
        }
    }
}

impl<M> JobMpsc<M> {
    pub fn start<F, O>(task: O) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
        O: FnOnce(mpsc::Receiver<M>) -> F,
    {
        let (tx, rx) = mpsc::channel(1024);
        Self::new(tx, task::spawn(task(rx)))
    }

    pub async fn send(&self, msg: M) -> Result<(), ()> {
        self.tx.send(msg).await.map_err(|_| ())
    }
}

impl<M> JobOne<M> {
    pub fn start<F, O>(task: O) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
        O: FnOnce(RO::Receiver<M>) -> F,
    {
        let (tx, rx) = RO::repeat_oneshot();
        Self::new(tx, task::spawn(task(rx)))
    }

    pub async fn send(&self, msg: M) -> Result<(), ()> {
        self.tx.send(msg).await.map_err(|_| ())
    }
}
