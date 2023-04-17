pub mod errormsg;
pub mod filesearch;
pub mod mpv;

#[protocol_macros::message_aggregator]
enum Front {
    None,
    Spotify,
    Mpv(mpv::Mpv),
    FileSearch(filesearch::FileSearch),
    PlayUrl,
    ErrorMsg(errormsg::ErrorMsg),
}
