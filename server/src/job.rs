use std::future::Future;
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

use crate::util::join_handle_wait;

// pub trait Job {
//     type State;
//     type Error;

//     fn next(&mut self) -> Result<State, Error>;
//     fn
// }

pub struct Job<T> {
    handle: Option<JoinHandle<()>>,
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

impl<T> Job<T> {
    fn new(tx: JobSender<T>, handle: JoinHandle<()>) -> Self {
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
                join_handle_wait(hand).await;
                handle.take();
            }
        }
    }

    pub fn start<F, O>(task: O) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
        O: FnOnce(JobReceiver<T>) -> F,
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
