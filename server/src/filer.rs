pub mod cache;
pub mod search;

use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use protocol::to_client::front::{self, filesearch};
use std::future::Future;
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

// TODO: lazy_static?
pub fn cache_file() -> PathBuf {
    config::cache_dir().join("files_cache")
}

// fn run(rx: TaskRcv, tx: StateSnd) -> FilerResult<()> {
//     let mut cache = read_cache(&cache_file())?;

//     loop {
//         match rx.blocking_recv() {
//             Err(_) => {
//                 log::info!("Filer task received exit signal");
//                 return Err(FilerError::Interrupted);
//             }
//             Ok(TaskMsg::Search(query)) => search::search(query, &cache, &tx)?,
//             Ok(TaskMsg::Cache) => cache = refresh_cache(&tx)?,
//             Ok(TaskMsg::BackToTheBeginning) => send_init(&cache, &tx)?,
//         }
//     }
// }

pub async fn refresh_cache<F, Fut>(prog_report: F) -> FilerResult<Cache>
where
    F: FnMut(filesearch::Refreshing) -> Fut,
    Fut: Future<Output = ()>,
{
    log::info!("Refreshing cache");
    let newcache =
        cache::refresh_cache(prog_report, config::root_dirs().to_vec()).await?;
    let newcache = cache::write_cache(&cache_file(), newcache).await?;
    log::info!("Refreshing cache done");
    Ok(newcache)
}

pub async fn refresh_cache_at_init() -> FilerResult<()> {
    refresh_cache(|_| async {}).await.map(|_| ())
}

pub async fn read_cache(cache_file: &Path) -> FilerResult<Cache> {
    let cache = match cache::read_cache(cache_file).await {
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
    Ok(cache)
}

fn send_init(cache: &Cache, tx: &StateSnd) -> FilerResult<()> {
    let init = filesearch::Init {
        last_cache_date: cache.updated(),
    };
    tx.blocking_send(Ok(init.into()))
        .map_err(|_| FilerError::Interrupted)
}
