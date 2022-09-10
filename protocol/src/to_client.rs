pub mod seat;

use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToClient {
    Seat(seat::Seat),
}

impl From<ToClient> for MessageKind {
    fn from(toclient: ToClient) -> MessageKind {
        MessageKind::ToClient(toclient)
    }
}
