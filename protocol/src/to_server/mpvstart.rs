message! {
    enum super::ToServer, MpvStart {
        Stop,
        File(String),
        Url(String),
    }
}
