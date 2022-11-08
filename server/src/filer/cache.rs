use std::{
    fs::{create_dir_all, File},
    path::Path,
    time::SystemTime,
};

use protocol::to_client::front::filesearch;
use walkdir::{DirEntry, WalkDir};

use super::StateSnd;

const EXT_WHITELIST: &[&str] = &[".mp4", ".mkv", ".wmv", ".webm", ".avi"];

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

    pub(super) fn full_path(&self) -> &str {
        &self.path
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

impl Default for Cache {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            updated: None,
            roots: Vec::new(),
        }
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

pub(super) fn read_cache(path: &Path) -> anyhow::Result<Cache> {
    let file = File::open(path)?;
    bincode::deserialize_from(file).map_err(|e| e.into())
}

pub(super) fn write_cache(path: &Path, contents: &Cache) -> anyhow::Result<()> {
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

pub(super) fn refresh_cache(tx: &StateSnd, roots: Vec<String>) -> Cache {
    let mut dirs: Vec<(usize, DirEntry)> = Vec::new();
    let mut files: Vec<CacheEntry> = Vec::new();

    let mut on_file =
        |de: DirEntry, i: usize| match de.into_path().into_os_string().into_string() {
            Ok(path) if has_whitelisted_extension(&path) => files.push(CacheEntry::new(
                path,
                i,
                roots
                    .get(i)
                    .expect("i is from enumerate, i.e. always in range")
                    .len(),
            )),
            Ok(_) => (),
            Err(path) => log::error!("Failed to convert '{:?} to a String", path),
        };

    {
        let total_roots = roots.len();
        for (i, root) in roots.iter().enumerate() {
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

    Cache::new(files, roots)
}

fn has_whitelisted_extension(path: &str) -> bool {
    EXT_WHITELIST.iter().any(|ext| path.ends_with(ext))
}

fn send_refreshing(tx: &StateSnd, i: usize, total: usize, exploding: bool) {
    let progress = progress(i, total);
    let msg = Ok(filesearch::FileSearch::Refreshing(
        filesearch::Refreshing::new(progress, exploding),
    ));

    tx.blocking_send(msg).ok();
}

fn progress(i: usize, total: usize) -> f64 {
    if total == 0 && i != 0 {
        0.0
    } else if i >= total {
        100.0
    } else {
        100.0 * (i as f64 / total as f64)
    }
}

#[test]
fn test_progress() {
    assert_eq!(progress(0, 0), 100.0);
    assert_eq!(progress(1, 0), 0.0);
    assert_eq!(progress(1, 1), 100.0);
    assert_eq!(progress(0, 1), 0.0);
    assert_eq!(progress(5, 10), 50.0);
}
