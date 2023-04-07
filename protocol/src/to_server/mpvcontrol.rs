message! {
    enum super::ToServer, MpvControl {
        TogglePause,
        SetAudio(i64),
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
        SetSub(i64),
        SubLarger,
        SubSmaller,
        SubMoveUp,
        SubMoveDown,
    }
}
