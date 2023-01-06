use std::{
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
    time::SystemTime,
};

use futures_util::{stream::FuturesUnordered, StreamExt};
use protocol::to_client::front::filesearch;
use walkdir::{DirEntry, WalkDir};

use super::{FilerError, FilerResult, StateSnd};

// TODO: move to config
const EXT_WHITELIST: &[&str] = &[".mp4", ".mkv", ".wmv", ".webm", ".avi"];

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub(super) struct Cache {
    files: Vec<CacheEntry>,
    updated: Option<SystemTime>,
    roots: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct CacheEntry {
    path: String,
    root: usize,
    root_len: usize,
}

impl CacheEntry {
    fn new(path: String, root: usize, root_len: usize) -> Self {
        Self {
            path,
            root,
            root_len,
        }
    }

    pub(super) fn root(&self) -> usize {
        self.root
    }

    pub(super) fn path_relative_root(&self) -> &str {
        &self.path[self.root_len..]
    }
}

impl AsRef<str> for CacheEntry {
    fn as_ref(&self) -> &str {
        self.path_relative_root()
    }
}

impl Cache {
    pub(super) fn new(files: Vec<CacheEntry>, roots: Vec<String>) -> Self {
        Self {
            files,
            updated: Some(SystemTime::now()),
            roots,
        }
    }

    pub(super) fn updated(&self) -> Option<SystemTime> {
        self.updated
    }

    pub(super) fn files(&self) -> &[CacheEntry] {
        &self.files
    }

    pub(super) fn is_outdated(&self, roots: &[String]) -> bool {
        self.roots != roots
    }
}

pub(super) fn read_cache(path: &Path) -> FilerResult<Cache> {
    let file = File::open(path)?;
    bincode::deserialize_from(file).map_err(|e| e.into())
}

pub(super) fn write_cache(path: &Path, contents: &Cache) -> FilerResult<()> {
    if let Some(p) = path.parent() {
        create_dir_all(p)?;
    }
    let mut file = File::create(path)?;

    bincode::serialize_into(&mut file, contents)?;

    file.sync_all()?;
    Ok(())
}

// TODO: run this on startup when some option in the config is turned on. Save a file in
// /tmp to signal that it has run once this boot. Support a tx that doesn't send anything.
pub(super) fn refresh_cache(tx: &StateSnd, roots: Vec<String>) -> FilerResult<Cache> {
    let mut num_errors = 0;
    let mut root_status: Vec<filesearch::RootStatus> = roots
        .iter()
        .map(|_| filesearch::RootStatus::Pending)
        .collect();

    log::info!("Refreshing cache... Probing roots...");
    probe(&roots, &mut root_status, tx)?;

    log::info!("Doing a shallow scan of available roots...");
    let shallow = shallow_scan(&roots, tx, &mut root_status)?;

    log::info!("Doing a deep scan of all roots...");
    let files = deep_scan(&shallow.dirs, tx, &roots, &root_status, &mut num_errors)?;

    log::info!("Creating a cache from all files...");
    let cache = create_cache_from_files(
        files,
        roots,
        num_errors,
        tx,
        shallow.files,
        shallow.dirs.len(),
        &root_status,
    )?;

    log::info!("Cache refresh done!");
    Ok(cache)
}

fn probe(
    roots: &[String],
    root_status: &mut [filesearch::RootStatus],
    tx: &StateSnd,
) -> FilerResult<()> {
    assert_eq!(roots.len(), root_status.len());
    root_status
        .iter_mut()
        .for_each(|s| *s = filesearch::RootStatus::Loading);
    send_refreshing(tx, 0, 0, &roots, &root_status, 0, false)?;

    tokio::runtime::Handle::current().block_on(async {
        let mut set: FuturesUnordered<_> = roots
            .iter()
            .enumerate()
            .map(|(i, root)| async move {
                let res =
                    tokio::fs::File::open([root, "."].iter().collect::<PathBuf>()).await;
                (i, res)
            })
            .collect();

        while let Some((i, res)) = set.next().await {
            root_status[i] = if res.is_err() {
                filesearch::RootStatus::Error
            } else {
                filesearch::RootStatus::Pending
            };
            send_refreshing_async(tx, 0, 0, &roots, &root_status, 0, false).await?;
        }
        Ok(())
    })
}

fn create_cache_from_files(
    files: Vec<(usize, DirEntry)>,
    roots: Vec<String>,
    mut num_errors: usize,
    tx: &StateSnd,
    files_shallow: Vec<(usize, DirEntry)>,
    num_dirs: usize,
    root_status: &[filesearch::RootStatus],
) -> Result<Cache, FilerError> {
    let mut cache_files: Vec<CacheEntry> = files_shallow
        .into_iter()
        .chain(files)
        .filter_map(|(i, de)| match create_cache_entry(de, i, &roots) {
            Ok(None) => None,
            Ok(Some(ce)) => Some(ce),
            Err(()) => {
                num_errors += 1;
                None
            }
        })
        .collect();

    send_refreshing(
        tx,
        num_dirs,
        num_dirs,
        &roots,
        root_status,
        num_errors,
        true,
    )?;

    cache_files
        .sort_unstable_by(|e1, e2| e1.path_relative_root().cmp(e2.path_relative_root()));
    Ok(Cache::new(cache_files, roots))
}

fn explode(
    path: &Path,
    mut on_file: impl FnMut(DirEntry),
    mut on_dir: impl FnMut(DirEntry),
) -> Result<(), walkdir::Error> {
    for de in WalkDir::new(path).max_depth(1).min_depth(1) {
        match de {
            Ok(e) if e.file_type().is_dir() => on_dir(e),
            Ok(e) if e.file_type().is_file() => on_file(e),
            Ok(e) => log::warn!("Found file of type '{:?}', ignoring...", e.file_type()),
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn create_cache_entry(
    de: DirEntry,
    i: usize,
    roots: &[String],
) -> Result<Option<CacheEntry>, ()> {
    match de.into_path().into_os_string().into_string() {
        Ok(path) if has_whitelisted_extension(&path) => Ok(Some(CacheEntry::new(
            path,
            i,
            roots
                .get(i)
                .expect("i is from enumerate, i.e. always in range")
                .len(),
        ))),
        Ok(_) => Ok(None),
        Err(path) => {
            log::error!("Failed to convert '{:?} to a String", path);
            Err(())
        }
    }
}

fn deep_scan(
    dirs: &[(usize, DirEntry)],
    tx: &StateSnd,
    roots: &[String],
    root_status: &[filesearch::RootStatus],
    num_errors: &mut usize,
) -> Result<Vec<(usize, DirEntry)>, FilerError> {
    let mut files: Vec<(usize, DirEntry)> = Vec::new();
    let total_dirs = dirs.len();

    for (i, (root, dir)) in dirs.iter().enumerate() {
        send_refreshing(tx, i, total_dirs, roots, root_status, *num_errors, false)?;

        for de in WalkDir::new(dir.path()) {
            match de {
                Err(e) => {
                    log::error!("Failed to walk: {}", e);
                    *num_errors += 1;
                }
                Ok(e) if e.file_type().is_file() => files.push((*root, e)),
                Ok(_) => (),
            }
        }
    }
    Ok(files)
}

struct ShallowScan {
    files: Vec<(usize, DirEntry)>,
    dirs: Vec<(usize, DirEntry)>,
}

fn shallow_scan(
    roots: &[String],
    tx: &StateSnd,
    root_status: &mut [filesearch::RootStatus],
) -> Result<ShallowScan, FilerError> {
    assert_eq!(roots.len(), root_status.len());
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for (i, root) in roots.iter().enumerate() {
        if root_status[i] == filesearch::RootStatus::Error {
            continue;
        }
        assert_eq!(root_status[i], filesearch::RootStatus::Pending);
        root_status[i] = filesearch::RootStatus::Loading;
        send_refreshing(tx, 0, dirs.len(), roots, &root_status, 0, false)?;
        match explode(
            root.as_ref(),
            |de| files.push((i, de)),
            |de| dirs.push((i, de)),
        ) {
            Err(e) => {
                root_status[i] = filesearch::RootStatus::Error;
                log::error!("Failed to walk '{}' cuz '{}'", root, e);
            }
            Ok(()) => root_status[i] = filesearch::RootStatus::Done,
        }
    }
    send_refreshing(tx, 0, dirs.len(), roots, &root_status, 0, false)?;

    Ok(ShallowScan { files, dirs })
}

fn has_whitelisted_extension(path: &str) -> bool {
    EXT_WHITELIST.iter().any(|ext| path.ends_with(ext))
}

fn send_refreshing(
    tx: &StateSnd,
    done_dirs: usize,
    total_dirs: usize,
    roots: &[String],
    root_status: &[filesearch::RootStatus],
    num_errors: usize,
    is_done: bool,
) -> FilerResult<()> {
    let msg = make_refreshing(
        roots,
        root_status,
        total_dirs,
        done_dirs,
        num_errors,
        is_done,
    );

    tx.blocking_send(Ok(msg.into()))
        .map_err(|_| FilerError::Interrupted)
}

async fn send_refreshing_async(
    tx: &StateSnd,
    done_dirs: usize,
    total_dirs: usize,
    roots: &[String],
    root_status: &[filesearch::RootStatus],
    num_errors: usize,
    is_done: bool,
) -> FilerResult<()> {
    let msg = make_refreshing(
        roots,
        root_status,
        total_dirs,
        done_dirs,
        num_errors,
        is_done,
    );

    tx.send(Ok(msg.into()))
        .await
        .map_err(|_| FilerError::Interrupted)
}

fn make_refreshing(
    roots: &[String],
    root_status: &[filesearch::RootStatus],
    total_dirs: usize,
    done_dirs: usize,
    num_errors: usize,
    is_done: bool,
) -> filesearch::Refreshing {
    assert_eq!(roots.len(), root_status.len());
    let msg = filesearch::Refreshing {
        roots: roots
            .iter()
            .zip(root_status)
            .map(|(path, status)| filesearch::RootInfo {
                path: path.to_string(),
                status: status.clone(),
            })
            .collect(),
        total_dirs,
        done_dirs,
        num_errors,
        is_done,
    };
    msg
}
