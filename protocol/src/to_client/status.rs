pub mod mpv;

message! {
    enum super::ToClient, Status {
        None,
        Spotify,
        Mpv(mpv::Mpv),
    }
}
