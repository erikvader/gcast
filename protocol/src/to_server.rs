use serde::{Deserialize, Serialize};

use crate::Message;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToServer {
    Ping(ping::Ping),
}

impl From<ToServer> for Message {
    fn from(toserver: ToServer) -> Self {
        Message::ToServer(toserver)
    }
}

pub mod ping {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Ping;
    impl From<Ping> for Message {
        fn from(ping: Ping) -> Self {
            ToServer::Ping(ping).into()
        }
    }
}
