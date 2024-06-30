use std::time::Duration;

use protocol_macros::message_part;

use crate::util::PositivePercent;

#[message_part]
struct PlayState {
    title: String,
    pause: bool,
    progress: Duration,
    length: Duration,
    volume: Option<PositivePercent>,
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
