use std::path::PathBuf;

message! {
    enum super::ToServer, MpvStart {
        Stop,
        File(PathBuf),
        Url(String),
    }
}
