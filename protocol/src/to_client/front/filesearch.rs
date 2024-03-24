pub mod init;
pub mod refreshing;
pub mod results;
pub mod tree;

use crate::to_client::ToClient;
use protocol_macros::message_aggregator;

#[message_aggregator(ToClient)]
enum FileSearch {
    Refreshing(refreshing::Refreshing),
    Results(results::Results),
    Tree(tree::Tree),
    Init(init::Init),
}
