pub mod front;
pub mod seat;

message! {
    enum super::MessageKind, ToClient {
        Seat(seat::Seat),
        Front(front::Front),
    }
}
