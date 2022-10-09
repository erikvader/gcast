message! {
    enum super::ToServer, MpvStart {
        Stop,
        File(String), // TODO: ta emot root index och en sträng
        Url(String),
    }
}
