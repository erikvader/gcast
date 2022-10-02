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
        SubLarger,
        SubSmaller,
        SubMoveUp,
        SubMoveDown,
    }
}
