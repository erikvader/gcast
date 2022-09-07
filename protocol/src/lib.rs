use serde::{Serialize,Deserialize};

#[derive(Debug,PartialEq,Eq,Clone,Serialize,Deserialize)]
pub enum Message {
    Noop
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MessageError(#[from] bincode::Error);

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, MessageError> {
        bincode::serialize(self).map_err(|e| e.into())
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, MessageError> {
        bincode::deserialize(bytes).map_err(|e| e.into())
    }
}

