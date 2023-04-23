use protocol::to_client::front::filesearch;

use crate::filer::cache::Cache;

const NUM_SEARCH_RESULTS: usize = 30;

pub fn search(query: String, cache: &Cache) -> filesearch::results::Results {
    log::info!("Searching for: {}", query);
    match searcher::search(&query, cache.files()) {
        Err(e) => {
            log::debug!(
                "failed to search, '{}' could not be compiled cuz: {}",
                query,
                e
            );
            filesearch::results::Results {
                results: Vec::new(),
                query,
                query_valid: false,
            }
        }
        Ok(mut res) => {
            let top = searcher::sorted_take(&mut res, NUM_SEARCH_RESULTS);
            log::debug!("Found {} results for '{}'", top.len(), query);
            let searchres = top
                .iter_mut()
                .map(|r| {
                    let c_entry = cache.files().get(r.get_index()).unwrap();
                    filesearch::results::SearchResult {
                        path: c_entry.path_relative_root().to_string(),
                        root: c_entry.root(),
                        indices: r.get_match().indices().to_vec(),
                        basename: c_entry.basename_char(),
                    }
                })
                .collect();

            filesearch::results::Results {
                results: searchres,
                query,
                query_valid: true,
            }
        }
    }
}
