mod run_filer;

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use protocol::{
    to_client::front::{self, filesearch::FileSearch},
    to_server::fscontrol::FsControl,
};
use tokio::sync::mpsc;

use crate::repeatable_oneshot;

static FILER_THREAD_ON: AtomicBool = AtomicBool::new(false);

pub type FilerResult<T> = Result<T, FilerError>;

#[derive(Debug, thiserror::Error)]
pub enum FilerError {
    #[error("Filer thread is already running")]
    AlreadyRunning,
    #[error("Filer::Exited: thread is not running anymore")]
    Exited,
}

type StateRcv = mpsc::Receiver<FilerResult<front::filesearch::FileSearch>>;
type StateSnd = mpsc::Sender<FilerResult<front::filesearch::FileSearch>>;
type SearchSnd = repeatable_oneshot::Sender<String>;
type SearchRcv = repeatable_oneshot::Receiver<String>;
type CacheSnd = repeatable_oneshot::Sender<()>;
type CacheRcv = repeatable_oneshot::Receiver<()>;

pub struct Handle {
    rx: StateRcv,
    tx: SearchSnd,
    cache_tx: CacheSnd,
}

impl Handle {
    pub async fn next(&mut self) -> FilerResult<front::filesearch::FileSearch> {
        self.rx.recv().await.unwrap_or(Err(FilerError::Exited))
    }

    pub async fn search(&self, query: String) -> FilerResult<()> {
        if self.tx.send(query).await.is_err() {
            return Err(FilerError::Exited);
        }
        Ok(())
    }

    pub async fn refresh_cache(&self) -> FilerResult<()> {
        if self.cache_tx.send(()).await.is_err() {
            return Err(FilerError::Exited);
        }
        Ok(())
    }
}

pub fn filer() -> FilerResult<Handle> {
    if FILER_THREAD_ON.swap(true, Ordering::SeqCst) {
        return Err(FilerError::AlreadyRunning);
    }

    let (h_tx, h_rx): (SearchSnd, _) = repeatable_oneshot::repeat_oneshot();
    let (c_tx, c_rx): (CacheSnd, _) = repeatable_oneshot::repeat_oneshot();
    let (s_tx, s_rx): (_, StateRcv) = mpsc::channel(crate::CHANNEL_SIZE);

    thread::spawn(move || {
        run_filer::run_filer(h_rx, s_tx, c_rx);
        FILER_THREAD_ON.store(false, Ordering::SeqCst);
    });

    Ok(Handle {
        rx: s_rx,
        tx: h_tx,
        cache_tx: c_tx,
    })
}
