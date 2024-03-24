// TODO: merge these into this file?
pub mod file;
pub mod url;

use crate::to_server::ToServer;
use protocol_macros::message_aggregator;

#[message_aggregator(ToServer)]
enum MpvStart {
    Stop,
    File(file::File),
    Url(url::Url),
}
