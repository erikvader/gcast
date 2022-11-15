use std::future::Future;
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

use crate::util::join_handle_wait;

pub mod handlejob;

pub struct Job<T, E> {
    handle: Option<JoinHandle<Result<(), E>>>,
    tx: Option<JobSender<T>>,
}

type JobSender<T> = mpsc::Sender<JobMsg<T>>;
type JobReceiver<T> = mpsc::Receiver<JobMsg<T>>;

pub enum JobMsg<T> {
    SendStatus,
    Ctrl(T),
}

#[derive(thiserror::Error, Debug)]
#[error("Job exited, can't send")]
pub struct JobExited;

impl<T, E> Job<T, E> {
    fn new(tx: JobSender<T>, handle: JoinHandle<Result<(), E>>) -> Self {
        Job {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    pub fn terminate(&mut self) {
        drop(self.tx.take());
    }

    pub async fn wait(&mut self) -> Result<(), E> {
        match &mut self.handle {
            None => Ok(()),
            Some(hand) => {
                let res = join_handle_wait(hand).await;
                self.handle.take();
                res
            }
        }
    }

    pub fn start<F, O>(task: O) -> Self
    where
        F: Future<Output = Result<(), E>> + Send + 'static,
        O: FnOnce(JobReceiver<T>) -> F,
        E: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(crate::CHANNEL_SIZE);
        Self::new(tx, task::spawn(task(rx)))
    }

    pub async fn send_ctrl(&self, msg: T) -> Result<(), JobExited> {
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

impl<T> JobMsg<T> {
    pub fn is_send_status(&self) -> bool {
        matches!(self, JobMsg::SendStatus)
    }
}
