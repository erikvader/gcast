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
        pub progress: u8, // [0-99]
        pub exploding: bool,
    }
}

message! {
    struct FileSearch, Results {
        pub results: Vec<String>, // TODO: String -> struct med String, index i config::root_dir, vec med index att highlighta
        pub query: String,
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
