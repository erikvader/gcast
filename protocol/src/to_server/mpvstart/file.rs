use protocol_macros::message_part;

#[message_part]
struct File {
    root: usize,
    path: String,
}
