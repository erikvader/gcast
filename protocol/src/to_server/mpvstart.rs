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
    // TODO: It's risky to allow the client to play an arbitrary filepath. Each file
    // option sent to the client should have some abstract id that represents it instead,
    // like the root has in this case.
    struct File {
        root: usize,
        path: String,
    }
}

pub mod url {
    #[protocol_macros::message_part]
    struct Url {
        // TODO: use an URL type
        url: String,
        paused: bool,
    }
}
