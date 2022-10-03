pub mod front;
pub mod notification;
pub mod seat;

message! {
    enum super::MessageKind, ToClient {
        Seat(seat::Seat),
        Front(front::Front),
        Notification(notification::Notification),
    }
}
