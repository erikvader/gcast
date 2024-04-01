use crate::to_server::ToServer;
use protocol_macros::message_aggregator;

#[message_aggregator(ToServer)]
enum MpvStart {
    Stop,
    File(file::File),
    Url(url::Url),
}

pub mod file {
    #[protocol_macros::message_part]
    struct File {
        root: usize,
        path: String,
    }
}

pub mod url {
    #[protocol_macros::message_part]
    struct Url {
        url: String,
        paused: bool,
    }
}
