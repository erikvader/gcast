pub mod init;
pub mod refreshing;
pub mod results;

use protocol_macros::message_aggregator;

#[message_aggregator]
enum FileSearch {
    Refreshing(refreshing::Refreshing),
    Results(results::Results),
    Init(init::Init),
}
