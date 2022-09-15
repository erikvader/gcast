pub mod seat;

use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToClient {
    Seat(seat::Seat),
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

impl From<ToClient> for MessageKind {
    fn from(toclient: ToClient) -> MessageKind {
        MessageKind::ToClient(toclient)
    }
}
