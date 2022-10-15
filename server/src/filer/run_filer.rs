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
    filer::cache::Cache,
    repeatable_oneshot::multiplex::{Either, MultiplexReceiver},
};

use super::{
    cache::{read_cache, refresh_cache, write_cache},
    StateSnd,
};

const NUM_SEARCH_RESULTS: usize = 20;

pub(super) fn run_filer(mut rx: MultiplexReceiver<String, ()>, tx: StateSnd) {
    let cache_file = config::cache_dir().join("files_cache");

    let mut cache = match read_cache(&cache_file) {
        Ok(c) if c.is_outdated(config::root_dirs()) => {
            log::info!("Saved cache is outdated");
            Cache::default()
        }
        Ok(c) => c,
        Err(e) => {
            match e.downcast_ref::<std::io::Error>() {
                Some(ioe) if ioe.kind() == ErrorKind::NotFound => {
                    log::info!("There is no cache yet")
                }
                _ => log::error!("Could not read cache file since: {:?}", e),
            }
            Cache::default()
        }
    };

    send_init(&cache, &tx);

    loop {
        match rx.blocking_recv() {
            None => {
                log::info!("Filer thread received exit signal");
                break;
            }
            Some(Either::Left(query)) => search(query, &cache, &tx),
            Some(Either::Right(())) => {
                log::info!("Refreshing cache");
                cache = refresh_cache(&tx, config::root_dirs().to_vec());
                if let Err(e) = write_cache(&cache_file, &cache) {
                    log::error!("Failed to write cache cuz: {:?}", e)
                }
                send_init(&cache, &tx);
                log::info!("Refreshing cache done");
            }
        }
    }
}

fn send_init(cache: &Cache, tx: &StateSnd) {
    let init = filesearch::Init {
        last_cache_date: cache.updated(),
    };
    tx.blocking_send(Ok(init.into())).ok();
}

fn search(query: String, cache: &Cache, tx: &StateSnd) {
    log::info!("Searching for: {}", query);
    match searcher::search(&query, cache.files()) {
        Err(e) => {
            log::debug!(
                "failed to search, '{}' could not be compiled cuz: {}",
                query,
                e
            );
            let res = filesearch::Results {
                results: Vec::new(),
                query,
                query_valid: false,
            };
            tx.blocking_send(Ok(res.into())).ok();
        }
        Ok(mut res) => {
            let top = searcher::sorted_take(&mut res, NUM_SEARCH_RESULTS);
            log::debug!("Found {} results for {}", top.len(), query);
            let searchres = top
                .into_iter()
                .map(|r| {
                    let c_entry = cache.files().get(r.get_index()).unwrap();
                    filesearch::SearchResult {
                        path: c_entry.path_relative_root().to_string(),
                        root: c_entry.root(),
                        indices: r.get_match().indices().to_vec(),
                    }
                })
                .collect();

            let res = filesearch::Results {
                results: searchres,
                query,
                query_valid: true,
            };
            tx.blocking_send(Ok(res.into())).ok();
        }
    }
}
