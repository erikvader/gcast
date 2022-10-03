use ordered_float::NotNan;

message! {
    enum super::Front, Mpv {
        Load,
        PlayState(PlayState),
    }
}

message! {
    struct Mpv, PlayState {
        pub pause: bool,
        pub progress: NotNan<f64>,
        pub length: NotNan<f64>,
        pub volume: NotNan<f64>,
        pub chapter: Option<(i64, i64)>,
    }
}

impl PlayState {
    pub fn new(
        pause: bool,
        progress: f64,
        length: f64,
        volume: f64,
        num_chapters: i64,
        chapter: i64,
    ) -> Self {
        Self {
            pause,
            progress: NotNan::new(progress).or(NotNan::new(0.0)).unwrap(),
            length: NotNan::new(length).or(NotNan::new(0.0)).unwrap(),
            volume: NotNan::new(volume).or(NotNan::new(0.0)).unwrap(),
            chapter: if num_chapters > 0 {
                Some((chapter, num_chapters))
            } else {
                None
            },
        }
    }
}
