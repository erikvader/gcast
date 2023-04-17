pub mod front;
pub mod seat;

#[protocol_macros::message_aggregator]
enum ToClient {
    Seat(seat::Seat),
    Front(front::Front),
}
