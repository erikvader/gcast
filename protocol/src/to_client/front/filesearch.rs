use std::time::SystemTime;

message! {
    enum super::Front, FileSearch {
        Refreshing(Refreshing),
        Results(Results),
        Init(Init),
    }
}

message! {
    struct FileSearch, Refreshing {
        pub progress: u8,
        pub exploding: bool,
    }
}

message! {
    struct FileSearch, Results {
        pub results: Vec<SearchResult>,
        pub query: String,
        pub query_valid: bool,
    }
}

message_part! {
    struct SearchResult {
        pub path: String,
        pub root: usize,
        pub indices: Vec<usize>
    }
}

message! {
    struct FileSearch, Init {
        pub last_cache_date: Option<SystemTime>,
    }
}

impl Default for FileSearch {
    fn default() -> Self {
        Self::Init(Init {
            last_cache_date: None,
        })
    }
}
