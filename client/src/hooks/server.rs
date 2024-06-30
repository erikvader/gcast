use super::visibility::use_page_visibility;
use super::websocket;
use super::websocket::use_websocket;
use crate::debug;
use derivative::Derivative;
use protocol::to_client;
use protocol::to_client::front::Front;
use protocol::to_client::seat::Seat;
use yew::hook;
use yew::use_effect_with;
use yew::use_state_eq;

#[derive(Clone, Derivative)]
#[derivative(PartialEq)]
pub struct UseServer {
    front: Front,
    connected: bool,
    accepted: Accepted,
    #[derivative(PartialEq = "ignore")]
    sender: websocket::Sender,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Accepted {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Clone)]
pub struct Sender(websocket::Sender);

impl UseServer {
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn is_disconnected(&self) -> bool {
        !self.is_connected()
    }

    pub fn front(&self) -> &Front {
        &self.front
    }

    pub fn accepted(&self) -> Accepted {
        self.accepted
    }

    pub fn sender(&self) -> Sender {
        Sender(self.sender.clone())
    }
}

impl Sender {
    pub fn send<T>(&self, msg: T)
    where
        T: protocol::ToServerable,
    {
        let msg = msg.to_server();
        log::debug!("Sending message: {msg:?}");

        let bytes = protocol::Message::from(msg).serialize();
        let Ok(bytes) = bytes else {
            log::error!("Failed to serialize: {}", bytes.unwrap_err());
            return;
        };

        let mut inner = self.0.clone();
        match inner.send(bytes) {
            Ok(()) => (),
            Err(()) => {
                log::error!("Failed to send a message, probably not connected yet")
            }
        }
    }
}

#[hook]
pub fn use_server() -> UseServer {
    // TODO: make the port configurable somehow
    let ws = use_websocket(1337);
    let visible = use_page_visibility();

    {
        let ws = ws.clone();
        use_effect_with(visible, move |visible| {
            if *visible {
                ws.open();
            } else {
                ws.close();
            }
        });
    }

    let front = use_state_eq(|| Front::None);
    let accepted = use_state_eq(|| Accepted::Pending);

    {
        let front = front.clone();
        let accepted = accepted.clone();
        let ws = ws.clone();
        let sender = Sender(ws.sender());
        use_effect_with(ws.message(), move |bytes| {
            if let Some(bytes) = bytes {
                match protocol::Message::deserialize(bytes) {
                    Err(e) => log::error!("Failed to deserialize message: {}", e),
                    Ok(protocol::Message::ToServer(msg)) => {
                        log::error!("Got a message for the server: {:?}", msg)
                    }
                    Ok(protocol::Message::ToClient(to_client::Front(new_front))) => {
                        front.set(new_front);
                    }
                    Ok(protocol::Message::ToClient(to_client::Seat(seat))) => {
                        match seat {
                            Seat::Accept => {
                                accepted.set(Accepted::Accepted);
                                sender.send(protocol::to_server::sendstatus::SendStatus);
                            }
                            Seat::Reject => {
                                accepted.set(Accepted::Rejected);
                            }
                        }
                    }
                }
            }
        });
    }

    UseServer {
        front: (*front).clone(),
        connected: ws.is_connected(),
        accepted: *accepted,
        sender: ws.sender(),
    }
}

#[hook]
pub fn use_server_debug(debug: &debug::Debug) -> UseServer {
    UseServer {
        front: debug.front(),
        connected: debug.is_connected(),
        accepted: debug.accepted(),
        sender: websocket::Sender::empty(),
    }
}
