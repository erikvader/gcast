macro_rules! into_ToClient {
    ($msg:ident) => {
        impl From<$msg> for MessageKind {
            fn from(m: $msg) -> MessageKind {
                ToClient::$msg(m).into()
            }
        }
    };
}

pub mod seat;
pub mod status;

use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToClient {
    Seat(seat::Seat),
    Status(status::Status),
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
