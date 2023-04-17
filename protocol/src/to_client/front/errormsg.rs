#[protocol_macros::message_part]
struct ErrorMsg {
    header: String,
    body: String,
}
