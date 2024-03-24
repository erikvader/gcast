use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    path::{Path, PathBuf},
};

use futures_util::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use protocol::to_client::front::filesearch;
use std::future::Future;
use tokio::task::spawn_blocking;
use walkdir::{DirEntry, WalkDir};

use crate::{
    filer::{
        cache::{CacheDirEntry, CacheEntry, Pointer},
        FilerError, FilerResult,
    },
    util::join_handle_wait_take,
};

use super::{Cache, CacheEntryBorrowed};

// TODO: move to config
const EXT_WHITELIST: &[&str] = &[".mp4", ".mkv", ".wmv", ".webm", ".avi"];

pub async fn read_cache(path: &Path) -> FilerResult<Cache> {
    // NOTE: tokio is doing this itself, i.e., creating a PathBuf
    // https://docs.rs/tokio/1.26.0/src/tokio/fs/read.rs.html#48-51
    let path = path.to_owned();
    join_handle_wait_take(spawn_blocking(move || {
        let file = File::open(&path)?;
        bincode::deserialize_from(file).map_err(|e| e.into())
    }))
    .await
}

pub async fn write_cache(path: &Path, contents: Cache) -> FilerResult<Cache> {
    // NOTE: Taking ownership of `contents` is only done to work around the 'static
    // requirement on `spawn_blocking`, use some kind of async variant of thread scopes
    // when available? Async bincode?
    // NOTE: tokio is doing this itself, i.e., creating a PathBuf
    // https://docs.rs/tokio/1.26.0/src/tokio/fs/read.rs.html#48-51
    let path = path.to_owned();
    join_handle_wait_take(spawn_blocking(move || {
        if let Some(p) = path.parent() {
            create_dir_all(p)?;
        }
        let mut file = File::create(path)?;

        bincode::serialize_into(&mut file, &contents)?;

        file.sync_all()?;
        Ok(contents)
    }))
    .await
}

pub async fn refresh_cache<F, Fut>(
    mut prog_report: F,
    roots: Vec<String>,
) -> FilerResult<Cache>
where
    F: FnMut(filesearch::refreshing::Refreshing) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut num_errors = 0;
    let mut root_status: Vec<filesearch::refreshing::RootStatus> = roots
        .iter()
        .map(|_| filesearch::refreshing::RootStatus::Pending)
        .collect();

    log::info!("Probing roots...");
    probe(&roots, &mut root_status, &mut prog_report).await?;

    log::info!("Doing a shallow scan of available roots...");
    let shallow = shallow_scan(&roots, &mut prog_report, &mut root_status).await?;

    log::info!("Doing a deep scan of all roots...");
    let deep = deep_scan(
        &shallow.dirs,
        &mut prog_report,
        &roots,
        &root_status,
        &mut num_errors,
    )
    .await?;

    log::info!("Creating a cache from all files...");
    let shallow_dirs_len = shallow.dirs.len();
    let cache = create_cache_from_files(
        deep.files.into_iter().chain(shallow.files),
        deep.dirs.into_iter().chain(shallow.dirs),
        roots,
        num_errors,
        &mut prog_report,
        shallow_dirs_len,
        &root_status,
    )
    .await?;

    log::info!("Cache refresh done!");
    Ok(cache)
}

async fn probe<F, Fut>(
    roots: &[String],
    root_status: &mut [filesearch::refreshing::RootStatus],
    mut prog_report: F,
) -> FilerResult<()>
where
    F: FnMut(filesearch::refreshing::Refreshing) -> Fut,
    Fut: Future<Output = ()>,
{
    assert_eq!(roots.len(), root_status.len());
    root_status
        .iter_mut()
        .for_each(|s| *s = filesearch::refreshing::RootStatus::Loading);

    prog_report(make_refreshing(0, 0, &roots, &root_status, 0, false)).await;

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
            filesearch::refreshing::RootStatus::Error
        } else {
            filesearch::refreshing::RootStatus::Pending
        };
        prog_report(make_refreshing(0, 0, &roots, &root_status, 0, false)).await;
    }
    Ok(())
}

async fn create_cache_from_files<F, Fut>(
    files: impl Iterator<Item = (usize, DirEntry)>,
    dirs: impl Iterator<Item = (usize, DirEntry)>,
    roots: Vec<String>,
    mut num_errors: usize,
    mut prog_report: F,
    num_dirs: usize,
    root_status: &[filesearch::refreshing::RootStatus],
) -> Result<Cache, FilerError>
where
    F: FnMut(filesearch::refreshing::Refreshing) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut cache_files: Vec<CacheEntry> = files
        .filter_map(|(i, de)| match create_cache_entry(de, i, &roots) {
            Ok(None) => None,
            Ok(Some(ce)) => Some(ce),
            Err(()) => {
                num_errors += 1;
                None
            }
        })
        .collect();

    let mut cache_dirs: Vec<CacheDirEntry> = dirs
        .filter_map(|(i, de)| match create_cache_dir_entry(de, i, &roots) {
            Ok(ce) => Some(ce),
            Err(()) => {
                num_errors += 1;
                None
            }
        })
        .collect();
    cache_dirs.extend(surface_scan(&roots));

    cache_files
        .sort_unstable_by(|e1, e2| e1.path_relative_root().cmp(e2.path_relative_root()));
    cache_dirs
        .sort_unstable_by(|e1, e2| e1.path_relative_root().cmp(e2.path_relative_root()));

    let (children, root_indices) = link(&cache_files, &cache_dirs);
    children.into_iter().for_each(|(i, pointers)| {
        cache_dirs
            .get_mut(i)
            .expect("the indices came from this vec")
            .set_children(pointers)
    });

    assert_eq!(
        roots.len(),
        root_indices.len(),
        "did not find the correct amount of roots"
    );

    let root_dir_pointers: Vec<Pointer> = root_indices
        .into_iter()
        .sorted_by(|l, r| {
            let l = cache_dirs.get(*l).expect("must exist").root();
            let r = cache_dirs.get(*r).expect("must exist").root();
            let l = roots.get(l).expect("must exist");
            let r = roots.get(r).expect("must exist");
            l.cmp(r)
        })
        .map(|i| Pointer::Dir(i))
        .collect();

    prog_report(make_refreshing(
        num_dirs,
        num_dirs,
        &roots,
        root_status,
        num_errors,
        true,
    ))
    .await;

    Ok(Cache::new(
        cache_files,
        cache_dirs,
        roots,
        root_dir_pointers,
    ))
}

fn link(
    files: &[CacheEntry],
    dirs: &[CacheDirEntry],
) -> (HashMap<usize, Vec<Pointer>>, Vec<usize>) {
    let mut roots = Vec::new();
    let dirs_inverted: HashMap<CacheEntryBorrowed, usize> = dirs
        .iter()
        .enumerate()
        .map(|(i, entry)| (entry.borrow_cache_entry(), i))
        .collect();
    let mut children: HashMap<CacheEntryBorrowed, Vec<Pointer>> = dirs
        .iter()
        .map(|entry| (entry.borrow_cache_entry(), vec![]))
        .collect();

    for (i, d) in dirs.iter().enumerate() {
        match d
            .parent()
            .as_ref()
            .and_then(|parent| children.get_mut(parent))
        {
            Some(pointers) => pointers.push(Pointer::Dir(i)),
            None => roots.push(i),
        }
    }

    for (i, f) in files.iter().enumerate() {
        match f
            .parent()
            .as_ref()
            .and_then(|parent| children.get_mut(parent))
        {
            Some(pointers) => pointers.push(Pointer::File(i)),
            None => log::error!(
                "The file '{:?}' does not have a parent directory for some reason",
                f
            ),
        }
    }

    let children: HashMap<usize, Vec<Pointer>> = children
        .into_iter()
        .map(|(path, pointers)| {
            (
                *dirs_inverted
                    .get(&path)
                    .expect("both maps have the same keys"),
                pointers,
            )
        })
        .collect();

    (children, roots)
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
            {
                let r = roots
                    .get(i)
                    .expect("i is from enumerate, i.e. always in range");
                path.strip_prefix(r)
                    .expect("Path must begin with this root")
                    .to_string()
            },
            i,
        ))),
        Ok(_) => Ok(None),
        Err(path) => {
            log::error!("Failed to convert '{:?} to a String", path);
            Err(())
        }
    }
}

fn create_cache_dir_entry(
    de: DirEntry,
    i: usize,
    roots: &[String],
) -> Result<CacheDirEntry, ()> {
    match de.into_path().into_os_string().into_string() {
        Ok(path) => Ok(CacheDirEntry::new(
            {
                let r = roots
                    .get(i)
                    .expect("i is from enumerate, i.e. always in range");
                path.strip_prefix(r)
                    .expect("Path must begin with this root")
                    .to_string()
            },
            i,
        )),
        Err(path) => {
            log::error!("Failed to convert '{:?} to a String", path);
            Err(())
        }
    }
}

struct Scan {
    files: Vec<(usize, DirEntry)>,
    dirs: Vec<(usize, DirEntry)>,
}

async fn deep_scan<F, Fut>(
    scan_in: &[(usize, DirEntry)],
    mut prog_report: F,
    roots: &[String],
    root_status: &[filesearch::refreshing::RootStatus],
    num_errors: &mut usize,
) -> Result<Scan, FilerError>
where
    F: FnMut(filesearch::refreshing::Refreshing) -> Fut,
    Fut: Future<Output = ()>,
{
    let mut files: Vec<(usize, DirEntry)> = Vec::new();
    let mut dirs: Vec<(usize, DirEntry)> = Vec::new();
    let total_dirs = scan_in.len();

    for (i, (root, dir)) in scan_in.iter().enumerate() {
        prog_report(make_refreshing(
            i,
            total_dirs,
            roots,
            root_status,
            *num_errors,
            false,
        ))
        .await;

        let root = *root;
        let dir = dir.path().to_owned(); // NOTE: annoying that this must be cloned :(
        let (new_errors, new_files, new_dirs) =
            join_handle_wait_take(spawn_blocking(move || {
                let mut new_errors = 0;
                let mut new_files = Vec::new();
                let mut new_dirs = Vec::new();

                for de in WalkDir::new(dir).min_depth(1) {
                    match de {
                        Err(e) => {
                            log::error!("Failed to walk: {}", e);
                            new_errors += 1;
                        }
                        Ok(e) if e.file_type().is_file() => new_files.push((root, e)),
                        Ok(e) if e.file_type().is_dir() => new_dirs.push((root, e)),
                        Ok(_) => (),
                    }
                }
                (new_errors, new_files, new_dirs)
            }))
            .await;

        *num_errors += new_errors;
        files.extend(new_files);
        dirs.extend(new_dirs);
    }

    Ok(Scan { files, dirs })
}

async fn shallow_scan<F, Fut>(
    roots: &[String],
    mut prog_report: F,
    root_status: &mut [filesearch::refreshing::RootStatus],
) -> Result<Scan, FilerError>
where
    F: FnMut(filesearch::refreshing::Refreshing) -> Fut,
    Fut: Future<Output = ()>,
{
    assert_eq!(roots.len(), root_status.len());
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for (i, root) in roots.iter().enumerate() {
        if root_status[i] == filesearch::refreshing::RootStatus::Error {
            continue;
        }
        assert_eq!(root_status[i], filesearch::refreshing::RootStatus::Pending);
        root_status[i] = filesearch::refreshing::RootStatus::Loading;
        prog_report(make_refreshing(
            0,
            dirs.len(),
            roots,
            &root_status,
            0,
            false,
        ))
        .await;

        match explode(
            root.as_ref(),
            |de| files.push((i, de)),
            |de| dirs.push((i, de)),
        ) {
            Err(e) => {
                root_status[i] = filesearch::refreshing::RootStatus::Error;
                log::error!("Failed to walk '{}' cuz '{}'", root, e);
            }
            Ok(()) => root_status[i] = filesearch::refreshing::RootStatus::Done,
        }
    }
    prog_report(make_refreshing(
        0,
        dirs.len(),
        roots,
        &root_status,
        0,
        false,
    ))
    .await;

    Ok(Scan { files, dirs })
}

fn surface_scan(roots: &[String]) -> Vec<CacheDirEntry> {
    roots
        .iter()
        .enumerate()
        .map(|(i, _path)| CacheDirEntry::new_root(i))
        .collect()
}

fn has_whitelisted_extension(path: &str) -> bool {
    EXT_WHITELIST.iter().any(|ext| path.ends_with(ext))
}

fn make_refreshing(
    done_dirs: usize,
    total_dirs: usize,
    roots: &[String],
    root_status: &[filesearch::refreshing::RootStatus],
    num_errors: usize,
    is_done: bool,
) -> filesearch::refreshing::Refreshing {
    assert_eq!(roots.len(), root_status.len());
    let msg = filesearch::refreshing::Refreshing {
        roots: roots
            .iter()
            .zip(root_status)
            .map(|(path, status)| filesearch::refreshing::RootInfo {
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
