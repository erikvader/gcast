pub mod playstate;

use crate::to_client::ToClient;
use protocol_macros::message_aggregator;

#[message_aggregator(ToClient)]
enum Mpv {
    Load,
    PlayState(playstate::PlayState),
}
