#[protocol_macros::message_part]
enum FsControl {
    Search(String),
    RefreshCache,
    BackToTheBeginning,
}
