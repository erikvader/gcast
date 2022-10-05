use std::future::Future;
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

pub struct Job<T> {
    handle: Option<JoinHandle<()>>,
    tx: Option<T>,
}

pub enum JobMsg<T> {
    SendStatus,
    Ctrl(T),
}

#[derive(thiserror::Error, Debug)]
#[error("Job exited, can't send")]
pub struct JobExited;

pub type JobMpsc<M> = Job<mpsc::Sender<JobMsg<M>>>;

impl<T> Job<T> {
    fn new(tx: T, handle: JoinHandle<()>) -> Self {
        Job {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    pub async fn terminate_wait(&mut self) {
        self.terminate();
        Self::impl_wait(&mut self.handle).await;
    }

    pub fn terminate(&mut self) {
        drop(self.tx.take());
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
        O: FnOnce(mpsc::Receiver<JobMsg<M>>) -> F,
    {
        let (tx, rx) = mpsc::channel(crate::CHANNEL_SIZE);
        Self::new(tx, task::spawn(task(rx)))
    }

    pub async fn send_ctrl(&self, msg: M) -> Result<(), JobExited> {
        self.tx
            .as_ref()
            .ok_or(JobExited)?
            .send(JobMsg::Ctrl(msg))
            .await
            .map_err(|_| JobExited)
    }

    pub async fn send_status(&self) -> Result<(), JobExited> {
        self.tx
            .as_ref()
            .ok_or(JobExited)?
            .send(JobMsg::SendStatus)
            .await
            .map_err(|_| JobExited)
    }
}
