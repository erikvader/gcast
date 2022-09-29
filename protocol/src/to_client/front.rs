pub mod mpv;

message! {
    enum super::ToClient, Front {
        None,
        Spotify,
        Mpv(mpv::Mpv),
        FileSearch, // list of search results, string that produced those results, last cache date, whether cache is updating and its progress
    }
}
