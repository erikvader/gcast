pub mod cache;
pub mod search;
pub mod tree;

use std::{
    io,
    path::{Path, PathBuf},
};

use protocol::to_client::front::filesearch;
use std::future::Future;

use crate::{config, filer::cache::Cache};

pub type FilerResult<T> = Result<T, FilerError>;

#[derive(Debug, thiserror::Error)]
pub enum FilerError {
    #[error("Failed to read the cache cuz: {0:?}")]
    IoError(#[from] io::Error),
    #[error("Failed to write to the cache cuz: {0:?}")]
    Bincode(#[from] bincode::Error),
}

// TODO: lazy_static?
pub fn cache_file() -> PathBuf {
    config::cache_dir().join("files_cache")
}

pub async fn refresh_cache<F, Fut>(prog_report: F) -> FilerResult<Cache>
where
    F: FnMut(filesearch::refreshing::Refreshing) -> Fut,
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
