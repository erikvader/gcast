pub mod to_client;
pub mod to_server;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Message {
    ToServer(to_server::ToServer),
    ToClient(to_client::ToClient),
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MessageError(#[from] bincode::Error);

impl Message {
    pub fn serialize(self) -> Result<Vec<u8>, MessageError> {
        bincode::serialize(&self).map_err(|e| e.into())
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, MessageError> {
        bincode::deserialize(bytes).map_err(|e| e.into())
    }

    pub fn to_client(self) -> Option<to_client::ToClient> {
        match self {
            Message::ToClient(toclient) => Some(toclient),
            Message::ToServer(_) => None,
        }
    }

    pub fn to_server(self) -> Option<to_server::ToServer> {
        match self {
            Message::ToServer(toserver) => Some(toserver),
            Message::ToClient(_) => None,
        }
    }
}
