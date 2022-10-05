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
    }
}

message! {
    struct FileSearch, Results {
        pub results: Vec<String>,
        pub query: String,
    }
}

message! {
    struct FileSearch, Init {
        pub last_cache_date: Option<()>, // TODO: some kind of date type
    }
}

impl Default for FileSearch {
    fn default() -> Self {
        Self::Init(Init {
            last_cache_date: None,
        })
    }
}
