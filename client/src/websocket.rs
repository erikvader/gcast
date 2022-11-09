use futures::{channel::mpsc, SinkExt, StreamExt};
use gloo_events::EventListener;
use gloo_net::websocket::{futures::WebSocket, Message as GlooMsg, WebSocketError};
use std::{collections::HashSet, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{use_bridge, Agent, Bridged, UseBridgeHandle};

pub struct WS {
    tx: mpsc::Sender<protocol::Message>,
    link: yew_agent::AgentLink<Self>,
    clients: HashSet<yew_agent::HandlerId>,
    connected: bool,
    #[allow(dead_code)] // This just needs to be kept alive, i.e. not dropped
    vischange: EventListener,
}

#[derive(Clone)]
pub enum WSOutput {
    Msg(Rc<protocol::Message>),
    Conn(bool),
}

pub enum WSInternal {
    Out(WSOutput),
    Visible(bool),
}

impl From<WSOutput> for WSInternal {
    fn from(out: WSOutput) -> Self {
        WSInternal::Out(out)
    }
}

impl Agent for WS {
    type Reach = yew_agent::Context<Self>;
    type Message = WSInternal;
    type Input = protocol::Message;
    type Output = WSOutput;

    fn create(link: yew_agent::AgentLink<Self>) -> Self {
        log::info!("Opening websocket connection");
        let window = web_sys::window().expect("could not access window");
        let hostname = window
            .location()
            .hostname()
            .unwrap_or_else(|e| panic!("could not get hostname: {:?}", e));

        // TODO: make this configurable somehow
        let ws = WebSocket::open(&format!("ws://{}:1337", hostname))
            .expect("only errors if url is bad?");

        let (mut tx, mut rx) = ws.split();
        let link2 = link.clone();
        spawn_local(async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    Ok(m) => {
                        if let Some(toclient) = try_to_client(&m) {
                            link2.send_message(WSOutput::Msg(Rc::new(toclient)));
                        }
                        link2.send_message(WSOutput::Conn(true));
                    }
                    Err(WebSocketError::ConnectionClose(e)) => {
                        log::warn!("Websocket disconnected: {:?}", e);
                        link2.send_message(WSOutput::Conn(false));
                    }
                    Err(e) => {
                        log::error!("Failed to read: {}", e);
                    }
                }
            }
            link2.send_message(WSOutput::Conn(false));
            log::info!("Websocket closed");
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
                    Ok(()) => {
                        // NOTE: This branch is taken if websocket is already closed for
                        // some reason.
                        // 'WebSocket is already in CLOSING or CLOSED state'
                        // link2.send_message(WSOutput::Conn(true));
                    }
                    Err(WebSocketError::ConnectionClose(e)) => {
                        log::info!("Websocket disconnected: {:?}", e);
                        link2.send_message(WSOutput::Conn(false));
                    }
                    Err(e) => log::error!("Failed to send: {}", e),
                }
            }
            link2.send_message(WSOutput::Conn(false));
            log::error!("WS ctx dropped");
        });

        let document = window.document().expect("could not access document");
        let link2 = link.clone();
        let vischange = EventListener::new(&window, "visibilitychange", move |_| {
            link2.send_message(WSInternal::Visible(
                document.visibility_state() == web_sys::VisibilityState::Visible,
            ));
        });

        WS {
            tx: ctx,
            link,
            clients: HashSet::new(),
            connected: false,
            vischange,
        }
    }

    fn destroy(&mut self) {
        todo!("close ws connection")
    }

    fn update(&mut self, msg: Self::Message) {
        let distribute = match msg {
            WSInternal::Out(WSOutput::Conn(conn)) if conn == self.connected => None,
            WSInternal::Out(WSOutput::Conn(conn)) => {
                self.connected = conn;
                Some(WSOutput::Conn(conn))
            }
            WSInternal::Visible(visible) => {
                // TODO:
                log::debug!("Hejsan: {}", visible);
                None
            }
            WSInternal::Out(m @ WSOutput::Msg(_)) => Some(m),
        };

        if let Some(m) = distribute {
            self.clients
                .iter()
                .for_each(|id| self.link.respond(*id, m.clone()));
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: yew_agent::HandlerId) {
        if let Err(e) = self.tx.try_send(msg) {
            log::error!("Failed to send to WS: {}", e);
        }
    }

    fn connected(&mut self, id: yew_agent::HandlerId) {
        self.clients.insert(id);
    }

    fn disconnected(&mut self, id: yew_agent::HandlerId) {
        self.clients.remove(&id);
    }
}

fn try_to_client(msg: &GlooMsg) -> Option<protocol::Message> {
    match msg {
        GlooMsg::Bytes(bytes) => match protocol::Message::deserialize(&bytes) {
            Err(e) => log::error!("Could not deserialize a message: {}", e),
            Ok(m) => {
                if m.is_to_client() {
                    return Some(m);
                } else {
                    log::warn!("Message not meant for client: {:?}", m);
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
    F: Fn(Rc<protocol::Message>) + 'static,
{
    use_bridge(move |wsout| {
        if let WSOutput::Msg(toclient) = wsout {
            on_output(toclient)
        }
    })
}

pub fn websocket_send<T>(msg: T)
where
    T: protocol::ToMessage,
{
    WS::bridge(yew::Callback::noop()).send(msg.to_message());
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
