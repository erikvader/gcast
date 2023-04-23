use ordered_float::NotNan;
use protocol_macros::message_part;

#[message_part]
struct PlayState {
    title: String,
    pause: bool,
    progress: NotNan<f64>,
    length: NotNan<f64>,
    volume: NotNan<f64>,
    chapter: Option<(i64, i64)>,
    subtitles: Vec<Track>,
    audios: Vec<Track>,
}

#[message_part]
struct Track {
    id: i64,
    title: String,
    selected: bool,
}
