use serde::{Deserialize, Serialize};

use crate::Message;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToClient {
    Pong(pong::Pong),
}

impl From<ToClient> for Message {
    fn from(toclient: ToClient) -> Message {
        Message::ToClient(toclient)
    }
}

pub mod pong {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Pong;
    impl From<Pong> for Message {
        fn from(pong: Pong) -> Message {
            ToClient::Pong(pong).into()
        }
    }
}
