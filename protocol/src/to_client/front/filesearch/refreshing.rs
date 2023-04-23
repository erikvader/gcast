use protocol_macros::{message_aggregator, message_part};

#[message_part]
struct Refreshing {
    roots: Vec<RootInfo>,
    total_dirs: usize,
    done_dirs: usize,
    is_done: bool,
    num_errors: usize,
}

#[message_part]
struct RootInfo {
    path: String,
    status: RootStatus,
}

#[message_aggregator]
enum RootStatus {
    Pending,
    Loading,
    Error,
    Done,
}
