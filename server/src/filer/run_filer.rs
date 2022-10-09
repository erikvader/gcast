use std::{
    fs::{create_dir_all, File},
    io::ErrorKind,
    path::Path,
    time::SystemTime,
};

use protocol::to_client::front::filesearch;
use walkdir::{DirEntry, WalkDir};

use crate::{
    config,
    repeatable_oneshot::multiplex::{Either, MultiplexReceiver},
};

use super::StateSnd;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Cache {
    files: Vec<CacheEntry>,
    updated: Option<SystemTime>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    path: String,
    root: usize,
}

impl CacheEntry {
    fn new(path: String, root: usize) -> Self {
        Self { path, root }
    }
}

impl AsRef<str> for CacheEntry {
    fn as_ref(&self) -> &str {
        &self.path
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            updated: None,
        }
    }
}

impl Cache {
    fn new(files: Vec<CacheEntry>) -> Self {
        Self {
            files,
            updated: Some(SystemTime::now()),
        }
    }
}

pub(super) fn run_filer(mut rx: MultiplexReceiver<String, ()>, tx: StateSnd) {
    let cache_file = config::cache_dir().join("files_cache");

    let mut cache = match read_cache(&cache_file) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Could not read cache file since: {:?}", e);
            Cache::default()
        }
    };

    tx.blocking_send(Ok(filesearch::FileSearch::Init(filesearch::Init {
        last_cache_date: cache.updated,
    })))
    .ok();

    loop {
        match rx.blocking_recv() {
            None => {
                log::info!("Filer thread received exit signal");
                break;
            }
            Some(Either::Left(query)) => todo!(),
            Some(Either::Right(())) => {
                cache = refresh_cache(&tx);
                match write_cache(&cache_file, &cache) {
                    Ok(()) => (),
                    Err(e) => log::error!("Failed to write cache cuz: {:?}", e),
                }
            }
        }
    }
}

fn read_cache(path: &Path) -> anyhow::Result<Cache> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Cache::default()),
        Err(e) => return Err(e.into()),
    };

    bincode::deserialize_from(file).map_err(|e| e.into())
}

fn write_cache(path: &Path, contents: &Cache) -> anyhow::Result<()> {
    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }
    let mut file = File::create(path)?;

    bincode::serialize_into(&mut file, contents)?;

    file.sync_all()?;
    Ok(())
}

fn all_files(dir: impl AsRef<Path>) -> impl Iterator<Item = DirEntry> {
    WalkDir::new(dir)
        .into_iter()
        .inspect(|res| {
            if let Err(e) = res {
                log::error!("Failed to walk: {}", e)
            }
        })
        .filter_map(|res| res.ok())
        .filter(|entry| entry.file_type().is_file())
}

fn explode(
    path: &Path,
    mut on_file: impl FnMut(DirEntry),
    mut on_dir: impl FnMut(DirEntry),
) {
    for de in WalkDir::new(path).max_depth(1).min_depth(1) {
        match de {
            Ok(e) if e.file_type().is_dir() => on_dir(e),
            Ok(e) if e.file_type().is_file() => on_file(e),
            Ok(e) => log::warn!("Found file of type '{:?}', ignoring...", e.file_type()),
            Err(e) => {
                log::error!("Failed to walk '{}' cuz '{}'", path.display(), e);
            }
        }
    }
}

fn refresh_cache(tx: &StateSnd) -> Cache {
    let mut dirs: Vec<(usize, DirEntry)> = Vec::new();
    let mut files: Vec<CacheEntry> = Vec::new();

    let mut on_file =
        |de: DirEntry, i: usize| match de.into_path().into_os_string().into_string() {
            Ok(path) => files.push(CacheEntry::new(path, i)),
            Err(path) => log::error!("Failed to convert '{:?} to a String", path),
        };

    {
        let total_roots = config::root_dirs().len();
        for (i, root) in config::root_dirs().iter().enumerate() {
            send_refreshing(tx, i, total_roots, true);
            explode(root.as_ref(), |de| on_file(de, i), |de| dirs.push((i, de)));
        }
        send_refreshing(tx, total_roots, total_roots, true);
    }

    {
        let total_dirs = dirs.len();
        for (i, (root, dir)) in dirs.into_iter().enumerate() {
            send_refreshing(tx, i, total_dirs, false);
            all_files(dir.path()).for_each(|de| on_file(de, root));
        }
        send_refreshing(tx, total_dirs, total_dirs, false);
    }

    Cache::new(files)
}

fn send_refreshing(tx: &StateSnd, i: usize, total: usize, exploding: bool) {
    let progress = progress(i, total);
    let msg = Ok(filesearch::FileSearch::Refreshing(filesearch::Refreshing {
        exploding,
        progress,
    }));

    tx.blocking_send(msg).ok();
}

fn progress(i: usize, total: usize) -> u8 {
    if total == 0 && i != 0 {
        0u8
    } else if i >= total {
        100u8
    } else {
        (100.0 * (i as f64 / total as f64)) as u8
    }
}

#[test]
fn test_progress() {
    assert_eq!(progress(0, 0), 100);
    assert_eq!(progress(1, 0), 0);
    assert_eq!(progress(1, 1), 100);
    assert_eq!(progress(0, 1), 0);
    assert_eq!(progress(5, 10), 50);
}
