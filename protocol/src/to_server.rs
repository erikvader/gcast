use serde::{Deserialize, Serialize};

use crate::{DeResult, Message, SerResult};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToServer {
    Ping(ping::Ping),
}

impl ToServer {
    fn serialize(self) -> SerResult {
        Message::ToServer(self).serialize()
    }

    pub fn deserialize(bytes: &[u8]) -> DeResult<Option<Self>> {
        match Message::deserialize(bytes)? {
            Message::ToServer(toserver) => Ok(Some(toserver)),
            Message::ToClient(_) => Ok(None),
        }
    }
}

pub mod ping {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Ping;
    impl Ping {
        pub fn serialize(self) -> SerResult {
            ToServer::Ping(self).serialize()
        }
    }
}
