use std::time::SystemTime;

use protocol_macros::{message_aggregator, message_part};

#[message_aggregator]
enum FileSearch {
    Refreshing(Refreshing),
    Results(Results),
    Init(Init),
}

#[message_part]
struct Refreshing {
    roots: Vec<RootInfo>,
    total_dirs: usize,
    done_dirs: usize,
    is_done: bool,
    num_errors: usize,
}

#[message_part]
struct RootInfo {
    path: String,
    status: RootStatus,
}

#[message_part]
enum RootStatus {
    Pending,
    Loading,
    Error,
    Done,
}

#[message_part]
struct Results {
    results: Vec<SearchResult>,
    query: String,
    query_valid: bool,
}

#[message_part]
struct SearchResult {
    path: String,
    root: usize,
    indices: Vec<usize>,
    basename: usize,
}

#[message_part]
struct Init {
    last_cache_date: Option<SystemTime>,
}
