pub mod errormsgctrl;
pub mod fscontrol;
pub mod fsstart;
pub mod mpvcontrol;
pub mod mpvstart;
pub mod playurlstart;
pub mod powerctrl;
pub mod sendstatus;
pub mod spotifyctrl;
pub mod spotifystart;

#[protocol_macros::message_aggregator]
enum ToServer {
    SendStatus(sendstatus::SendStatus),
    PowerCtrl(powerctrl::PowerCtrl),
    MpvControl(mpvcontrol::MpvControl),
    MpvStart(mpvstart::MpvStart),
    SpotifyStart(spotifystart::SpotifyStart),
    SpotifyCtrl(spotifyctrl::SpotifyCtrl),
    FsStart(fsstart::FsStart),
    FsControl(fscontrol::FsControl),
    PlayUrlStart(playurlstart::PlayUrlStart),
    ErrorMsgCtrl(errormsgctrl::ErrorMsgCtrl),
}
