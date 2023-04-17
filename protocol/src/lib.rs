use protocol_macros::message_aggregator;
use protocol_macros::message_part;

// TODO: remove
macro_rules! message {
    (enum $kind:ty, $name:ident $body:tt) => {
        pub use $name::*;
        message! {@x enum $kind, $name $body}
    };
    (struct $kind:ty, $name:ident $body:tt) => {
        message! {@x struct $kind, $name $body}
    };
    (@x $enumstruct:ident $kind:ty, $name:ident $body:tt) => {
        #[protocol_macros::message_part]
        $enumstruct $name $body

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
pub mod util;

use std::sync::atomic::AtomicU64;

pub type Id = u64;

static MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[message_aggregator]
enum MessageKind {
    ToServer(to_server::ToServer),
    ToClient(to_client::ToClient),
}

#[message_part]
struct Message {
    id: Id,
    kind: MessageKind,
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct MessageError(#[from] bincode::Error);

// TODO: reworka vart Message skapas.
// Endast delarna som tar hand om att skicka och ta emot meddelandena (typ connections.rs)
// ska skapa dem och hantera att ID:n kommer i rätt ordning osv. Ta bort den atomiska
// message countern. Varje ny anslutning ska börja om på 0 på båda sidor.
// Applikaitonsdelarna ska skicka och ta emot ToServer och ToClient UTAN att gör om dem
// till ett Message först, eller ens bry sig om att den structen finns. Kanske göra
// mottagardelen i Client mer uppenbar och inte bara en funktion i huvudkomponenten? Man
// ska aldrig behöva kolla ID:t när man skickar, för den ska alltid skapas precis innan
// man skickar. Bara på mottagning skall ID kollas, och det gör inte servern i dagsläget.
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

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn is_to_server(&self) -> bool {
        matches!(self.kind, MessageKind::ToServer(_))
    }

    pub fn is_to_client(&self) -> bool {
        matches!(self.kind, MessageKind::ToClient(_))
    }

    pub fn is_newer_than(&self, old: Id) -> bool {
        self.id() > old
    }

    pub fn is_expected_or_newer_than(&self, expected: Id) -> bool {
        self.id() >= expected
    }
}

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
