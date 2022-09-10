use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Seat {
    Accept,
    Reject,
}

impl From<Seat> for MessageKind {
    fn from(seat: Seat) -> MessageKind {
        ToClient::Seat(seat).into()
    }
}

impl Seat {
    pub fn is_accecpted(&self) -> bool {
        *self == Seat::Accept
    }
}
