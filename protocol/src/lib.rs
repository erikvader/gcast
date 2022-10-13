macro_rules! message_part {
    ($($rest:tt)+) => {
        #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
        pub $($rest)+
    };
}

macro_rules! message {
    (enum $kind:ty, $name:ident $body:tt) => {
        pub use $name::*;
        message! {@x enum $kind, $name $body}
    };
    (struct $kind:ty, $name:ident $body:tt) => {
        message! {@x struct $kind, $name $body}
    };
    (@x $enumstruct:ident $kind:ty, $name:ident $body:tt) => {
        message_part! {
            $enumstruct $name $body
        }

        impl From<$name> for $crate::Message {
            fn from(m: $name) -> Self {
                <$kind>::$name(m).into()
            }
        }

        impl From<$name> for $kind {
            fn from(m: $name) -> Self {
                <$kind>::$name(m)
            }
        }
    };
}

pub mod to_client;
pub mod to_server;

use std::sync::atomic::AtomicU64;

static MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

message_part! {
    enum MessageKind {
        ToServer(to_server::ToServer),
        ToClient(to_client::ToClient),
    }
}

message_part! {
    struct Message {
        id: u64,
        kind: MessageKind,
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MessageError(#[from] bincode::Error);

impl Message {
    fn new(kind: MessageKind) -> Self {
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

    pub fn take_to_client(self) -> to_client::ToClient {
        match self.kind {
            MessageKind::ToClient(toclient) => toclient,
            MessageKind::ToServer(_) => panic!("tried to take ToClient on a ToServer"),
        }
    }

    pub fn take_to_server(self) -> to_server::ToServer {
        match self.kind {
            MessageKind::ToServer(toserver) => toserver,
            MessageKind::ToClient(_) => panic!("tried to take ToServer on a ToClient"),
        }
    }

    pub fn borrow_to_client(&self) -> &to_client::ToClient {
        match &self.kind {
            MessageKind::ToClient(toclient) => toclient,
            MessageKind::ToServer(_) => panic!("tried to borrow ToClient on a ToServer"),
        }
    }

    pub fn borrow_to_server(&self) -> &to_server::ToServer {
        match &self.kind {
            MessageKind::ToServer(toserver) => toserver,
            MessageKind::ToClient(_) => panic!("tried to borrow ToServer on a ToClient"),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn is_to_server(&self) -> bool {
        match self.kind {
            MessageKind::ToServer(_) => true,
            _ => false,
        }
    }

    pub fn is_to_client(&self) -> bool {
        match self.kind {
            MessageKind::ToClient(_) => true,
            _ => false,
        }
    }
}

// impl<K> From<K> for Message
// where
//     K: Into<MessageKind>,
// {
//     fn from(kind: K) -> Self {
//         Message::new(kind.into())
//     }
// }

impl From<MessageKind> for Message {
    fn from(mk: MessageKind) -> Self {
        Message::new(mk)
    }
}

pub trait ToMessage {
    fn to_message(self) -> Message;
}

impl<T> ToMessage for T
where
    T: Into<Message>,
{
    fn to_message(self) -> Message {
        self.into()
    }
}

impl AsRef<to_client::ToClient> for Message {
    fn as_ref(&self) -> &to_client::ToClient {
        self.borrow_to_client()
    }
}

impl AsRef<to_server::ToServer> for Message {
    fn as_ref(&self) -> &to_server::ToServer {
        self.borrow_to_server()
    }
}
