message! {
    enum super::ToClient, Seat {
        Accept,
        Reject,
    }
}

impl Seat {
    pub fn is_accepted(&self) -> bool {
        *self == Seat::Accept
    }
}
