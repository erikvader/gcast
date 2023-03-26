use std::sync::Arc;

use tokio::sync::{Mutex, Notify};

#[derive(thiserror::Error, Debug)]
#[error("Other end closed, nowhere to send or nothing to receive")]
pub struct OtherEndClosed;

pub struct Sender<T> {
    data: Option<Arc<Mutex<Option<T>>>>,
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
            data: Some(Arc::clone(&data)),
            notifier: Arc::clone(&notifier),
        },
        Receiver { data, notifier },
    )
}

impl<T> Sender<T>
where
    T: std::fmt::Debug,
{
    pub async fn send_test_and_set<F>(
        &self,
        test_and_set: F,
    ) -> Result<(), OtherEndClosed>
    where
        F: FnOnce(Option<&T>) -> Option<T>,
    {
        if self.is_closed() {
            return Err(OtherEndClosed);
        }
        let mut place = self
            .data
            .as_ref()
            .expect("is only None after a drop")
            .lock()
            .await;
        if let Some(newvalue) = test_and_set(place.as_ref()) {
            if let Some(old) = place.replace(newvalue) {
                log::trace!("Replaced '{:?}' with new data in a rep_oneshot", old);
            }
        }
        self.notifier.notify_one();
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn send(&self, msg: T) -> Result<(), OtherEndClosed> {
        self.send_test_and_set(|_| Some(msg)).await
    }

    pub fn is_closed(&self) -> bool {
        Arc::strong_count(self.data.as_ref().unwrap()) == 1
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // NOTE: Make sure the receiver doesn't hang if this is dropped between
        // `is_closed` and `notified`.
        drop(self.data.take());
        self.notifier.notify_one();
    }
}

impl<T> Receiver<T> {
    pub async fn recv(&self) -> Result<T, OtherEndClosed> {
        loop {
            if self.is_closed() {
                break Err(OtherEndClosed);
            }
            self.notifier.notified().await;
            if let Some(x) = self.data.lock().await.take() {
                break Ok(x);
            }
        }
    }

    #[allow(dead_code)]
    pub fn blocking_recv(&self) -> Result<T, OtherEndClosed> {
        tokio::runtime::Handle::current().block_on(self.recv())
    }

    pub fn is_closed(&self) -> bool {
        Arc::strong_count(&self.data) == 1
    }
}
