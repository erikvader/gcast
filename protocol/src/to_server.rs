pub mod sendstatus;

use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToServer {
    SendStatus(sendstatus::SendStatus),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ServerMsg {
    toserver: ToServer,
    id: u64,
}

impl ServerMsg {
    pub(super) fn new(id: u64, toserver: ToServer) -> Self {
        Self { id, toserver }
    }

    pub fn server_kind(&self) -> &ToServer {
        &self.toserver
    }

    pub fn take(self) -> ToServer {
        self.toserver
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}

impl From<ToServer> for MessageKind {
    fn from(toserver: ToServer) -> Self {
        MessageKind::ToServer(toserver)
    }
}
