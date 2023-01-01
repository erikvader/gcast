use std::{
    fs::{create_dir_all, File},
    path::Path,
    time::SystemTime,
};

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

// TODO: this is multiple functions
pub(super) fn refresh_cache(tx: &StateSnd, roots: Vec<String>) -> FilerResult<Cache> {
    let mut dirs: Vec<(usize, DirEntry)> = Vec::new();
    let mut files: Vec<CacheEntry> = Vec::new();
    let mut num_errors: usize = 0;
    let mut root_status: Vec<filesearch::RootStatus> = roots
        .iter()
        .map(|_| filesearch::RootStatus::Pending)
        .collect();

    let mut on_file = |de: DirEntry, i: usize, ne: &mut usize| match de
        .into_path()
        .into_os_string()
        .into_string()
    {
        Ok(path) if has_whitelisted_extension(&path) => files.push(CacheEntry::new(
            path,
            i,
            roots
                .get(i)
                .expect("i is from enumerate, i.e. always in range")
                .len(),
        )),
        Ok(_) => (),
        Err(path) => {
            *ne += 1;
            log::error!("Failed to convert '{:?} to a String", path);
        }
    };

    {
        assert!(roots.len() == root_status.len());
        // TODO: parallelize with rayon or something?
        for (i, root) in roots.iter().enumerate() {
            root_status[i] = filesearch::RootStatus::Loading;
            send_refreshing(tx, 0, dirs.len(), &roots, &root_status, num_errors, false)?;
            match explode(
                root.as_ref(),
                |de| on_file(de, i, &mut num_errors),
                |de| dirs.push((i, de)),
            ) {
                Err(e) => {
                    root_status[i] = filesearch::RootStatus::Error;
                    log::error!("Failed to walk '{}' cuz '{}'", root, e);
                }
                Ok(()) => root_status[i] = filesearch::RootStatus::Done,
            }
        }
        send_refreshing(tx, 0, dirs.len(), &roots, &root_status, num_errors, false)?;
    }

    {
        let total_dirs = dirs.len();
        for (i, (root, dir)) in dirs.into_iter().enumerate() {
            send_refreshing(tx, i, total_dirs, &roots, &root_status, num_errors, false)?;
            for de in WalkDir::new(dir.path()) {
                match de {
                    Err(e) => {
                        log::error!("Failed to walk: {}", e);
                        num_errors += 1;
                    }
                    Ok(e) if e.file_type().is_file() => on_file(e, root, &mut num_errors),
                    Ok(_) => (),
                }
            }
        }
        send_refreshing(
            tx,
            total_dirs,
            total_dirs,
            &roots,
            &root_status,
            num_errors,
            true,
        )?;
    }

    files.sort_unstable_by(|e1, e2| e1.path_relative_root().cmp(e2.path_relative_root()));
    Ok(Cache::new(files, roots))
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

    tx.blocking_send(Ok(msg.into()))
        .map_err(|_| FilerError::Interrupted)
}
