use protocol_macros::message_part;

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
