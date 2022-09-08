pub mod to_client;
pub mod to_server;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
enum Message {
    ToServer(to_server::ToServer),
    ToClient(to_client::ToClient),
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MessageError(#[from] bincode::Error);

pub type DeResult<T> = std::result::Result<T, MessageError>;
pub type SerResult = DeResult<Vec<u8>>;

impl Message {
    fn serialize(self) -> SerResult {
        bincode::serialize(&self).map_err(|e| e.into())
    }

    fn deserialize(bytes: &[u8]) -> DeResult<Self> {
        bincode::deserialize(bytes).map_err(|e| e.into())
    }
}
