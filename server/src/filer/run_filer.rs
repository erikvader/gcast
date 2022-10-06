use std::{
    fs::{create_dir_all, read_to_string, File},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    time::SystemTime,
};

use walkdir::{DirEntry, WalkDir};

use crate::repeatable_oneshot::multiplex::{Either, MultiplexReceiver};

use super::StateSnd;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Cache {
    files: Vec<String>,
    updated: SystemTime,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            updated: std::time::UNIX_EPOCH,
        }
    }
}

impl Cache {
    fn new(files: Vec<String>) -> Self {
        Self {
            files,
            updated: SystemTime::now(),
        }
    }
}

pub(super) fn run_filer(mut rx: MultiplexReceiver<String, ()>, tx: StateSnd) {
    let cache_dir = PathBuf::from("~/.cache/gcast");
    let cache_file = cache_dir.join("file_cache");
    let conf_dir = PathBuf::from("~/.config/gcast");
    let conf_file = conf_dir.join("movie_dirs");

    let mut cache = match read_cache(&cache_file) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Could not read cache file since: {:?}", e);
            Cache::default()
        }
    };

    loop {
        match rx.blocking_recv() {
            None => {
                log::info!("Filer thread received exit signal");
                break;
            }
            Some(Either::Left(query)) => todo!(),
            Some(Either::Right(())) => {
                refresh_cache(&conf_file, &cache_file);
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

fn all_files(
    dirs: impl IntoIterator<Item = impl AsRef<Path>>,
) -> impl Iterator<Item = String> {
    dirs.into_iter()
        .flat_map(|p| WalkDir::new(p))
        .inspect(|res| {
            if let Err(e) = res {
                log::error!("Failed to walk: {}", e)
            }
        })
        .filter_map(|res| res.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|file| {
            file.path()
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| file)
        })
        .inspect(|res| {
            if let Err(file) = res {
                log::error!("Failed to convert '{:?}' to a String", file);
            }
        })
        .filter_map(|res| res.ok())
}

fn read_config(path: &Path) -> io::Result<Vec<String>> {
    Ok(read_to_string(path)?
        .lines()
        .map(|s| s.to_string())
        .collect())
}

fn explode_all(dirs: &[&Path]) -> (Vec<DirEntry>, Vec<DirEntry>) {
    todo!()
}

// TODO: make this report progress
fn refresh_cache(config_file: &Path, cache_file: &Path) -> anyhow::Result<Cache> {
    todo!()
}
