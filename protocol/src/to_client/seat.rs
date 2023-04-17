#[protocol_macros::message_part]
enum Seat {
    Accept,
    Reject,
}

impl Seat {
    pub fn is_accepted(&self) -> bool {
        *self == Seat::Accept
    }
}
