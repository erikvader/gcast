use protocol_macros::{message_aggregator, message_part};

#[message_aggregator]
enum MpvStart {
    Stop,
    File(File),
    Url(Url),
}

#[message_part]
struct File {
    root: usize,
    path: String,
}

#[message_part]
struct Url(String);
