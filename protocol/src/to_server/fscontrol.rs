#[protocol_macros::message_aggregator]
#[no_intos]
enum FsControl {
    Search(String),
    RefreshCache,
    BackToTheBeginning,
}
