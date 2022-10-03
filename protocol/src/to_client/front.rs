pub mod filesearch;
pub mod mpv;

message! {
    enum super::ToClient, Front {
        None,
        Spotify,
        Mpv(mpv::Mpv),
        FileSearch(filesearch::FileSearch),
    }
}
