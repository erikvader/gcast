use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Seat {
    Accept,
    Reject,
}

into_ToClient!(Seat);

impl Seat {
    pub fn is_accepted(&self) -> bool {
        *self == Seat::Accept
    }
}
