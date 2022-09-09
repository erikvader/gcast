use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToServer {
    Ping(ping::Ping),
}

impl From<ToServer> for MessageKind {
    fn from(toserver: ToServer) -> Self {
        MessageKind::ToServer(toserver)
    }
}

pub mod ping {
    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
    pub struct Ping;
    impl From<Ping> for MessageKind {
        fn from(ping: Ping) -> Self {
            ToServer::Ping(ping).into()
        }
    }
}
