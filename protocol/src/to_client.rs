use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToClient {
    Pong(pong::Pong),
}

impl From<ToClient> for MessageKind {
    fn from(toclient: ToClient) -> MessageKind {
        MessageKind::ToClient(toclient)
    }
}

pub mod pong {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Pong;
    impl From<Pong> for MessageKind {
        fn from(pong: Pong) -> MessageKind {
            ToClient::Pong(pong).into()
        }
    }
}
