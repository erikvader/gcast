use std::sync::Arc;

use tokio::sync::{mpsc, Mutex, Notify};

#[derive(thiserror::Error, Debug)]
#[error("Other end closed, nowhere to send or nothing to receive")]
pub struct OtherEndClosed;

pub struct Sender<T> {
    data: Arc<Mutex<Option<T>>>,
    notifier: Arc<Notify>,
}

pub struct Receiver<T> {
    data: Arc<Mutex<Option<T>>>,
    notifier: Arc<Notify>,
}

// Maybe better described as a replacing queue of size 1
pub fn repeat_oneshot<T>() -> (Sender<T>, Receiver<T>) {
    let data = Arc::new(Mutex::new(None));
    let notifier = Arc::new(Notify::new());
    (
        Sender {
            data: Arc::clone(&data),
            notifier: Arc::clone(&notifier),
        },
        Receiver { data, notifier },
    )
}

impl<T> Sender<T> {
    pub async fn send(&self, msg: T) -> Result<(), OtherEndClosed> {
        if self.is_closed() {
            return Err(OtherEndClosed);
        }
        self.data.lock().await.replace(msg);
        self.notifier.notify_one();
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        Arc::strong_count(&self.data) == 1
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // NOTE: Make sure the receiver doesn't hang if this is dropped between
        // `is_closed` and `notified`.
        self.notifier.notify_one();
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&self) -> Result<T, OtherEndClosed> {
        if self.is_closed() {
            return Err(OtherEndClosed);
        }
        self.notifier.notified().await;
        self.data.lock().await.take().ok_or(OtherEndClosed)
    }

    pub fn is_closed(&self) -> bool {
        Arc::strong_count(&self.data) == 1
    }
}

pub mod multiplex {
    use super::*;

    pub enum Either<T1, T2> {
        Left(T1),
        Right(T2),
    }

    pub struct MultiplexReceiver<T1, T2> {
        rx: mpsc::Receiver<Either<T1, T2>>,
        notify: Arc<Notify>,
    }

    impl<T1, T2> MultiplexReceiver<T1, T2> {
        pub fn blocking_recv(&mut self) -> Option<Either<T1, T2>> {
            self.notify.notify_one();
            self.rx.blocking_recv()
        }
    }

    pub fn multiplex<T1, T2>(
        left: Receiver<T1>,
        right: Receiver<T2>,
    ) -> MultiplexReceiver<T1, T2>
    where
        T1: Send + 'static,
        T2: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1);
        let notify = Arc::new(Notify::new());
        let notify2 = Arc::clone(&notify);

        tokio::spawn(async move {
            let mut l = Box::pin(left.recv());
            let mut r = Box::pin(right.recv());
            loop {
                notify2.notified().await;

                tokio::select! {
                    x = &mut l => {
                        match x {
                            Err(_) => break,
                            Ok(res) => if tx.send(Either::Left(res)).await.is_err() {
                                break
                            }
                        }
                        l = Box::pin(left.recv());
                    }
                    x = &mut r => {
                        match x {
                            Err(_) => break,
                            Ok(res) => if tx.send(Either::Right(res)).await.is_err() {
                                break
                            }
                        }
                        r = Box::pin(right.recv());
                    }
                }
            }
        });

        MultiplexReceiver { rx, notify }
    }
}
