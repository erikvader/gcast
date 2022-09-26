use std::sync::Arc;

use tokio::sync::{Mutex, Notify};

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
