use ordered_float::NotNan;

message! {
    enum super::Front, Mpv {
        Load,
        PlayState(PlayState),
    }
}

message! {
    struct Mpv, PlayState {
        pause: bool,
        progress: NotNan<f64>,
        length: NotNan<f64>,
        volume: NotNan<f64>,
        chapter: Option<(i64, i64)>,
    }
}
