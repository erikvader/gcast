mod cache;
mod search;

use std::{
    io,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

use protocol::to_client::front::{self, filesearch};
use tokio::{
    sync::mpsc,
    task::{spawn_blocking, JoinHandle},
};

use crate::{
    config, filer::cache::Cache, repeatable_oneshot, util::join_handle_wait_take,
};

static FILER_THREAD_ON: AtomicBool = AtomicBool::new(false);

pub type FilerResult<T> = Result<T, FilerError>;

#[derive(Debug, thiserror::Error)]
pub enum FilerError {
    #[error("Filer thread is already running")]
    AlreadyRunning,
    #[error("Filer::Exited: thread is not running anymore")]
    Exited,
    #[error("Failed to read the cache cuz: {0:?}")]
    IoError(#[from] io::Error),
    #[error("Failed to write to the cache cuz: {0:?}")]
    Bincode(#[from] bincode::Error),
    #[error("Interrupted by user")]
    Interrupted,
}

#[derive(Debug)]
enum TaskMsg {
    Cache,
    Search(String),
    BackToTheBeginning,
}

type StateRcv = mpsc::Receiver<FilerResult<front::filesearch::FileSearch>>;
type StateSnd = mpsc::Sender<FilerResult<front::filesearch::FileSearch>>;
type TaskSnd = repeatable_oneshot::Sender<TaskMsg>;
type TaskRcv = repeatable_oneshot::Receiver<TaskMsg>;

pub struct Handle {
    rx: StateRcv,
    tx: TaskSnd,
    joinhandle: JoinHandle<()>,
}

impl Handle {
    pub async fn next(&mut self) -> FilerResult<front::filesearch::FileSearch> {
        self.rx.recv().await.unwrap_or(Err(FilerError::Exited))
    }

    pub async fn search(&self, query: String) -> FilerResult<()> {
        if self
            .tx
            .send_test_and_set(|old| match old {
                Some(TaskMsg::Cache) | Some(TaskMsg::BackToTheBeginning) => None,
                None | Some(TaskMsg::Search(_)) => Some(TaskMsg::Search(query)),
            })
            .await
            .is_err()
        {
            return Err(FilerError::Exited);
        }
        Ok(())
    }

    pub async fn refresh_cache(&self) -> FilerResult<()> {
        if self
            .tx
            // TODO: solve this by using a priority number on each enum variant instead
            .send_test_and_set(|old| match old {
                Some(TaskMsg::BackToTheBeginning) => None,
                None | Some(TaskMsg::Search(_)) | Some(TaskMsg::Cache) => {
                    Some(TaskMsg::Cache)
                }
            })
            .await
            .is_err()
        {
            return Err(FilerError::Exited);
        }
        Ok(())
    }

    pub async fn back_to_the_beginning(&self) -> FilerResult<()> {
        if self.tx.send(TaskMsg::BackToTheBeginning).await.is_err() {
            return Err(FilerError::Exited);
        }
        Ok(())
    }

    pub async fn wait_until_closed(self) {
        drop(self.rx);
        drop(self.tx);
        join_handle_wait_take(self.joinhandle).await;
    }
}

fn run(rx: TaskRcv, tx: StateSnd) -> FilerResult<()> {
    let cache_file = config::cache_dir().join("files_cache");
    let mut cache = read_cache(&tx, &cache_file)?;

    loop {
        match rx.blocking_recv() {
            Err(_) => {
                log::info!("Filer task received exit signal");
                return Err(FilerError::Interrupted);
            }
            Ok(TaskMsg::Search(query)) => search::search(query, &cache, &tx)?,
            Ok(TaskMsg::Cache) => cache = refresh_cache(&tx, &cache_file)?,
            Ok(TaskMsg::BackToTheBeginning) => send_init(&cache, &tx)?,
        }
    }
}

fn refresh_cache(tx: &StateSnd, cache_file: &Path) -> FilerResult<Cache> {
    log::info!("Refreshing cache");
    let newcache = cache::refresh_cache(tx, config::root_dirs().to_vec())?;
    cache::write_cache(cache_file, &newcache)?;
    log::info!("Refreshing cache done");
    Ok(newcache)
}

fn read_cache(tx: &StateSnd, cache_file: &Path) -> FilerResult<Cache> {
    let cache = match cache::read_cache(cache_file) {
        Ok(c) if c.is_outdated(config::root_dirs()) => {
            log::info!("Saved cache is outdated");
            Ok(Cache::default())
        }
        Ok(c) => Ok(c),
        Err(FilerError::IoError(ioe)) if ioe.kind() == io::ErrorKind::NotFound => {
            log::info!("There is no cache yet");
            Ok(Cache::default())
        }
        Err(e) => Err(e),
    }?;
    send_init(&cache, tx)?;
    Ok(cache)
}

fn send_init(cache: &Cache, tx: &StateSnd) -> FilerResult<()> {
    let init = filesearch::Init {
        last_cache_date: cache.updated(),
    };
    tx.blocking_send(Ok(init.into()))
        .map_err(|_| FilerError::Interrupted)
}

pub fn filer() -> FilerResult<Handle> {
    if FILER_THREAD_ON.swap(true, Ordering::SeqCst) {
        return Err(FilerError::AlreadyRunning);
    }

    let (t_tx, t_rx): (TaskSnd, TaskRcv) = repeatable_oneshot::repeat_oneshot();
    let (s_tx, s_rx): (StateSnd, StateRcv) = mpsc::channel(crate::CHANNEL_SIZE);

    let joinhandle = spawn_blocking(move || {
        if let Err(e) = run(t_rx, s_tx.clone()) {
            s_tx.blocking_send(Err(e)).ok();
        }

        FILER_THREAD_ON.store(false, Ordering::SeqCst);
    });

    Ok(Handle {
        rx: s_rx,
        tx: t_tx,
        joinhandle,
    })
}
