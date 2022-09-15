use futures::{channel::mpsc, SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message as GlooMsg, WebSocketError};
use protocol::to_client::ToClient;
use std::{collections::HashSet, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{use_bridge, Agent, UseBridgeHandle};

pub struct WS {
    tx: mpsc::Sender<protocol::Message>,
    link: yew_agent::AgentLink<Self>,
    clients: HashSet<yew_agent::HandlerId>,
    connected: bool,
}

#[derive(Clone)]
pub enum WSOutput {
    Msg(u64, Rc<ToClient>),
    Conn(bool),
}

impl Agent for WS {
    type Reach = yew_agent::Context<Self>;
    type Message = Self::Output;
    type Input = protocol::Message;
    type Output = WSOutput;

    fn create(link: yew_agent::AgentLink<Self>) -> Self {
        log::info!("Opening websocket connection");
        let ws =
            WebSocket::open("ws://localhost:1337").expect("only errors if url is bad?");

        let (mut tx, mut rx) = ws.split();
        let link2 = link.clone();
        spawn_local(async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    Ok(m) => {
                        log::info!("Received: {:?}", m);
                        if let Some((id, toclient)) = try_to_client(&m) {
                            link2.send_message(WSOutput::Msg(id, Rc::new(toclient)));
                        }
                        link2.send_message(WSOutput::Conn(true));
                    }
                    Err(WebSocketError::ConnectionClose(e)) => {
                        log::warn!("websocket disconnected: {:?}", e);
                        link2.send_message(WSOutput::Conn(false));
                    }
                    Err(e) => {
                        log::error!("failed to read: {}", e);
                    }
                }
            }
            link2.send_message(WSOutput::Conn(false));
            log::info!("websocket closed");
        });

        let (ctx, mut crx) = mpsc::channel::<Self::Input>(1000);
        let link2 = link.clone();
        spawn_local(async move {
            while let Some(msg) = crx.next().await {
                let bytes = match msg.serialize() {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        log::error!("Could not serialize cuz: {}", e);
                        continue;
                    }
                };

                match tx.send(GlooMsg::Bytes(bytes)).await {
                    Ok(()) => link2.send_message(WSOutput::Conn(true)),
                    Err(WebSocketError::ConnectionClose(e)) => {
                        log::info!("websocket disconnected: {:?}", e);
                        link2.send_message(WSOutput::Conn(false));
                    }
                    Err(e) => log::error!("failed to send: {}", e),
                }
            }
            link2.send_message(WSOutput::Conn(false));
            log::error!("WS ctx dropped");
        });

        WS {
            tx: ctx,
            link,
            clients: HashSet::new(),
            connected: false,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            WSOutput::Conn(conn) if conn == self.connected => return,
            WSOutput::Conn(conn) => self.connected = conn,
            _ => (),
        }
        self.clients
            .iter()
            .for_each(|id| self.link.respond(*id, msg.clone()));
    }

    fn handle_input(&mut self, msg: Self::Input, _id: yew_agent::HandlerId) {
        if let Err(e) = self.tx.try_send(msg) {
            log::error!("failed to send to WS: {}", e);
        }
    }

    fn connected(&mut self, id: yew_agent::HandlerId) {
        log::debug!("An agent connected {:?}", id);
        self.clients.insert(id);
    }

    fn disconnected(&mut self, id: yew_agent::HandlerId) {
        log::debug!("An agent disconnected {:?}", id);
        self.clients.remove(&id);
    }
}

fn try_to_client(msg: &GlooMsg) -> Option<(u64, ToClient)> {
    match msg {
        GlooMsg::Bytes(bytes) => match protocol::Message::deserialize(&bytes) {
            Err(e) => log::error!("Could not deserialize a message: {}", e),
            Ok(m) => {
                let id = m.id();
                if let Some(toclient) = m.try_to_client() {
                    return Some((id, toclient));
                } else {
                    log::warn!("message not meant for client");
                }
            }
        },
        GlooMsg::Text(_) => {
            log::warn!("Received a text message from the server")
        }
    }
    None
}

pub fn use_websocket<F>(on_output: F) -> UseBridgeHandle<WS>
where
    F: Fn(u64, Rc<ToClient>) + 'static,
{
    use_bridge(move |wsout| {
        if let WSOutput::Msg(id, toclient) = wsout {
            on_output(id, toclient)
        }
    })
}

pub fn use_websocket_status<F>(on_change: F) -> UseBridgeHandle<WS>
where
    F: Fn(bool) + 'static,
{
    use_bridge(move |wsout| {
        if let WSOutput::Conn(b) = wsout {
            on_change(b)
        }
    })
}
