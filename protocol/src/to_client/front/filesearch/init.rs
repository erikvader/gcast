use std::time::SystemTime;

use protocol_macros::message_part;

#[message_part]
struct Init {
    last_cache_date: Option<SystemTime>,
}
