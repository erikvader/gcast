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
        roots: Vec<RootInfo>,
        total_dirs: usize,
        done_dirs: usize,
        is_done: bool,
        num_errors: usize,
    }
}

message_part! {
    struct RootInfo {
        path: String,
        status: RootStatus,
    }
}

message_part! {
    enum RootStatus {
        Pending,
        Loading,
        Error,
        Done,
    }
}

message! {
    struct FileSearch, Results {
        results: Vec<SearchResult>,
        query: String,
        query_valid: bool,
    }
}

message_part! {
    struct SearchResult {
        path: String,
        root: usize,
        indices: Vec<usize>,
    }
}

message! {
    struct FileSearch, Init {
        last_cache_date: Option<SystemTime>,
    }
}
