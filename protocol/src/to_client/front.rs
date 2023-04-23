pub mod errormsg;
pub mod filesearch;
pub mod mpv;

use crate::to_client::ToClient;

#[protocol_macros::message_aggregator(ToClient)]
enum Front {
    None,
    Spotify,
    Mpv(mpv::Mpv),
    FileSearch(filesearch::FileSearch),
    PlayUrl,
    ErrorMsg(errormsg::ErrorMsg),
}
