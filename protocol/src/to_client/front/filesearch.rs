pub mod init;
pub mod refreshing;
pub mod results;

use crate::to_client::ToClient;
use protocol_macros::message_aggregator;

#[message_aggregator(ToClient)]
enum FileSearch {
    Refreshing(refreshing::Refreshing),
    Results(results::Results),
    Init(init::Init),
}
