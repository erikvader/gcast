message! {
    enum super::ToServer, MpvStart {
        Stop,
        File(File),
        Url(String),
    }
}

message! {
    struct MpvStart, File {
        root: usize,
        path: String,
    }
}
