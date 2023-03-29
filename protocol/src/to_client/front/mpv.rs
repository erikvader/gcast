use ordered_float::NotNan;

message! {
    enum super::Front, Mpv {
        Load,
        PlayState(PlayState),
    }
}

message! {
    struct Mpv, PlayState {
        title: String,
        pause: bool,
        progress: NotNan<f64>,
        length: NotNan<f64>,
        volume: NotNan<f64>,
        chapter: Option<(i64, i64)>,
        subtitles: Vec<Track>,
        audios: Vec<Track>,
    }
}

message_part! {
    struct Track {
        id: i64,
        title: String,
        selected: bool,
    }
}
