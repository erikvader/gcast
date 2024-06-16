use crate::util::Percent;

#[protocol_macros::message_aggregator]
#[no_intos]
enum MpvControl {
    TogglePause,
    SetAudio(i64),
    VolumeUp,
    VolumeDown,
    ToggleMute,
    SubDelayEarlier,
    SubDelayLater,
    NextChapter,
    PrevChapter,
    SeekAbs(Percent),
    SeekBack,
    SeekForward,
    SeekBackLong,
    SeekForwardLong,
    SetSub(i64),
    SubLarger,
    SubSmaller,
    SubMoveUp,
    SubMoveDown,
}
