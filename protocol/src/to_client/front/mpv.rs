use ordered_float::NotNan;

use crate::util::not_nan_or_zero;

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
            progress: not_nan_or_zero(progress),
            length: not_nan_or_zero(length),
            volume: not_nan_or_zero(volume),
            chapter: if num_chapters > 0 {
                Some((chapter, num_chapters))
            } else {
                None
            },
        }
    }
}
