pub mod fscontrol;
pub mod fsstart;
pub mod mpvcontrol;
pub mod mpvstart;
pub mod sendstatus;
pub mod spotifystart;

message! {
    enum super::MessageKind, ToServer {
        SendStatus(sendstatus::SendStatus),
        MpvControl(mpvcontrol::MpvControl),
        MpvStart(mpvstart::MpvStart),
        SpotifyStart(spotifystart::SpotifyStart),
        FsStart(fsstart::FsStart),
        FsControl(fscontrol::FsControl),
    }
}
