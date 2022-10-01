use crate::repeatable_oneshot as RO;
use std::future::Future;
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

pub struct Job<T> {
    handle: Option<JoinHandle<()>>,
    tx: T,
}

#[derive(thiserror::Error, Debug)]
#[error("Job exited, can't send")]
pub struct JobExited;

pub type JobMpsc<M> = Job<mpsc::Sender<M>>;
pub type JobOne<M> = Job<RO::Sender<M>>;

impl<T> Job<T> {
    fn new(tx: T, handle: JoinHandle<()>) -> Self {
        Job {
            tx,
            handle: Some(handle),
        }
    }

    pub async fn terminate_wait(mut self) {
        drop(self.tx);
        Self::impl_wait(&mut self.handle).await;
    }

    pub async fn wait(&mut self) {
        Self::impl_wait(&mut self.handle).await;
    }

    async fn impl_wait(handle: &mut Option<JoinHandle<()>>) {
        match handle {
            None => (),
            Some(hand) => {
                match hand.await {
                    Err(err) if err.is_panic() => {
                        std::panic::resume_unwind(err.into_panic())
                    }
                    Err(err) if err.is_cancelled() => {
                        unreachable!("There is no way to call abort on the handle")
                    }
                    Err(_) => unreachable!("a new variant got introduced"),
                    Ok(()) => (),
                }
                *handle = None;
            }
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

    pub async fn send(&self, msg: M) -> Result<(), JobExited> {
        self.tx.send(msg).await.map_err(|_| JobExited)
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

    pub async fn send(&self, msg: M) -> Result<(), JobExited> {
        self.tx.send(msg).await.map_err(|_| JobExited)
    }
}
