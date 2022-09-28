use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum MpvControl {
    TogglePause,
    Quit,
    CycleAudio,
    VolumeUp,
    VolumeDown,
    ToggleMute,
    SubDelayEarlier,
    SubDelayLater,
    NextChapter,
    PrevChapter,
    SeekBack,
    SeekForward,
    SeekBackLong,
    SeekForwardLong,
    CycleSub,
    SubLarger,
    SubSmaller,
    SubMoveUp,
    SubMoveDown,
}

into_ToServer!(MpvControl);
