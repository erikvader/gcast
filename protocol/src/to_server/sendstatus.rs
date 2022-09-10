use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]

pub struct SendStatus;

impl From<SendStatus> for MessageKind {
    fn from(sendstatus: SendStatus) -> MessageKind {
        ToServer::SendStatus(sendstatus).into()
    }
}
