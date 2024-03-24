#[protocol_macros::message_aggregator]
enum FsStart {
    Start,
    Stop,
    RefreshCache,
    Search,
    Tree,
}
