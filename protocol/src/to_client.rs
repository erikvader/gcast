use serde::{Deserialize, Serialize};

use crate::{DeResult, Message, SerResult};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToClient {
    Pong(pong::Pong),
}

impl ToClient {
    fn serialize(self) -> SerResult {
        Message::ToClient(self).serialize()
    }

    pub fn deserialize(bytes: &[u8]) -> DeResult<Option<Self>> {
        match Message::deserialize(bytes)? {
            Message::ToClient(toclient) => Ok(Some(toclient)),
            Message::ToServer(_) => Ok(None),
        }
    }
}

pub mod pong {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Pong;
    impl Pong {
        pub fn serialize(self) -> SerResult {
            ToClient::Pong(self).serialize()
        }
    }
}
