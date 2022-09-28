use ordered_float::NotNan;

use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Mpv {
    Load,
    Play(PlayState),
}

into_Status!(Mpv);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct PlayState {
    pause: bool,
    progress: NotNan<f64>,
    length: NotNan<f64>,
    volume: NotNan<f64>,
    chapter: Option<(i64, i64)>,
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
