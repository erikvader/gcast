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
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerMsg {
    toserver: ToServer,
    id: u64,
}

impl ServerMsg {
    pub(super) fn new(id: u64, toserver: ToServer) -> Self {
        Self { id, toserver }
    }

    pub fn server_kind(&self) -> &ToServer {
        &self.toserver
    }

    pub fn take(self) -> ToServer {
        self.toserver
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
