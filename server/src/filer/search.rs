use protocol::to_client::front::filesearch;

use crate::filer::cache::Cache;

use super::StateSnd;

const NUM_SEARCH_RESULTS: usize = 30;

pub(super) fn search(query: String, cache: &Cache, tx: &StateSnd) {
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
            log::debug!("Found {} results for '{}'", top.len(), query);
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
