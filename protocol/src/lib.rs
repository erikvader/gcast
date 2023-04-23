use protocol_macros::message_aggregator;

pub mod to_client;
pub mod to_server;
pub mod util;

#[message_aggregator]
enum Message {
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
}

pub trait ToServerable {
    fn to_server(self) -> to_server::ToServer;
}

pub trait ToClientable {
    fn to_client(self) -> to_client::ToClient;
}

impl<T> ToServerable for T
where
    T: Into<to_server::ToServer>,
{
    fn to_server(self) -> to_server::ToServer {
        self.into()
    }
}

impl<T> ToClientable for T
where
    T: Into<to_client::ToClient>,
{
    fn to_client(self) -> to_client::ToClient {
        self.into()
    }
}
