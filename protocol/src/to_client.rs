pub mod seat;
pub mod status;

message! {
    enum super::MessageKind, ToClient {
        Seat(seat::Seat),
        Status(status::Status),
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientMsg {
    toclient: ToClient,
    id: u64,
}

impl ClientMsg {
    pub(super) fn new(id: u64, toclient: ToClient) -> Self {
        Self { id, toclient }
    }

    pub fn client_kind(&self) -> &ToClient {
        &self.toclient
    }

    pub fn take(self) -> ToClient {
        self.toclient
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
