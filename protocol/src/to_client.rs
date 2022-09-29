pub mod seat;
pub mod status;

message! {
    enum super::MessageKind, ToClient {
        Seat(seat::Seat),
        Status(status::Status),
    }
}
