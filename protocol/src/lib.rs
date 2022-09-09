pub mod to_client;
pub mod to_server;

use std::sync::atomic::AtomicU64;

use serde::{Deserialize, Serialize};

const MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum MessageKind {
    ToServer(to_server::ToServer),
    ToClient(to_client::ToClient),
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Message {
    id: u64,
    kind: MessageKind,
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MessageError(#[from] bincode::Error);

impl Message {
    pub fn new(kind: MessageKind) -> Self {
        Message {
            id: MESSAGE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            kind,
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, MessageError> {
        bincode::serialize(&self).map_err(|e| e.into())
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, MessageError> {
        bincode::deserialize(bytes).map_err(|e| e.into())
    }

    pub fn to_client(self) -> Option<to_client::ToClient> {
        match self.kind {
            MessageKind::ToClient(toclient) => Some(toclient),
            MessageKind::ToServer(_) => None,
        }
    }

    pub fn to_server(self) -> Option<to_server::ToServer> {
        match self.kind {
            MessageKind::ToServer(toserver) => Some(toserver),
            MessageKind::ToClient(_) => None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<K> From<K> for Message
where
    K: Into<MessageKind>,
{
    fn from(kind: K) -> Self {
        Message::new(kind.into())
    }
}
