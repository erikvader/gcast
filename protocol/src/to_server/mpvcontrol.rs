message! {
    enum super::ToServer, MpvControl {
        TogglePause,
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
        SetSub(i64),
        SubLarger,
        SubSmaller,
        SubMoveUp,
        SubMoveDown,
    }
}
