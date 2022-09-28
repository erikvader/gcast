use std::path::PathBuf;

use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MpvPlay {
    Stop,
    File(PathBuf),
    Url(String),
}

into_ToServer!(MpvPlay);
