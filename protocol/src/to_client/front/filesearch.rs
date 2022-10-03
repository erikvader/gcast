message! {
    enum super::Front, FileSearch {
        Refreshing(Refreshing),
        Results(Results),
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
        pub last_cache_date: (), // TODO: some kind of date type
        pub query: String,
    }
}
