use std::time::SystemTime;

use ordered_float::NotNan;

use crate::util::not_nan_or_zero;

message! {
    enum super::Front, FileSearch {
        Refreshing(Refreshing),
        Results(Results),
        Init(Init),
    }
}

message! {
    struct FileSearch, Refreshing {
        progress: NotNan<f64>, // [0, 100]
        exploding: bool,
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

impl Default for FileSearch {
    fn default() -> Self {
        Self::Init(Init {
            last_cache_date: None,
        })
    }
}

impl Refreshing {
    pub fn new(progress: f64, exploding: bool) -> Self {
        Self {
            exploding,
            progress: not_nan_or_zero(progress),
        }
    }
}
