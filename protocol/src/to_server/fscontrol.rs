message! {
    enum super::ToServer, FsControl {
        Search(String),
        RefreshCache,
        BackToTheBeginning,
    }
}
