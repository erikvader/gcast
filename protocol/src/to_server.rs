pub mod errormsgctrl;
pub mod fscontrol;
pub mod fsstart;
pub mod mpvcontrol;
pub mod mpvstart;
pub mod sendstatus;
pub mod spotifyctrl;
pub mod spotifystart;

message! {
    enum super::MessageKind, ToServer {
        SendStatus(sendstatus::SendStatus),
        MpvControl(mpvcontrol::MpvControl),
        MpvStart(mpvstart::MpvStart),
        SpotifyStart(spotifystart::SpotifyStart),
        SpotifyCtrl(spotifyctrl::SpotifyCtrl),
        FsStart(fsstart::FsStart),
        FsControl(fscontrol::FsControl),
        ErrorMsgCtrl(errormsgctrl::ErrorMsgCtrl),
    }
}
