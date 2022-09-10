pub mod sendstatus;

use serde::{Deserialize, Serialize};

use crate::MessageKind;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToServer {
    SendStatus(sendstatus::SendStatus),
}

impl From<ToServer> for MessageKind {
    fn from(toserver: ToServer) -> Self {
        MessageKind::ToServer(toserver)
    }
}
