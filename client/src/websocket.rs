use futures::{
    channel::{mpsc, oneshot},
    join, select,
    stream::{SplitSink, SplitStream},
    FutureExt, SinkExt, StreamExt,
};
use gloo_events::EventListener;
use gloo_net::websocket::{futures::WebSocket, Message as GlooMsg, WebSocketError};
use std::{collections::HashSet, rc::Rc};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{use_bridge, Agent, Bridged, UseBridgeHandle};

pub struct WS {
    link: yew_agent::AgentLink<Self>,
    clients: HashSet<yew_agent::HandlerId>,
    connected: bool,
    _vischange: EventListener,
    connection: Option<Connection>,
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

struct Connection {
    ctx: mpsc::Sender<protocol::Message>,
    close_rx: oneshot::Sender<()>,
    rx_end: oneshot::Receiver<SplitStream<WebSocket>>,
    tx_end: oneshot::Receiver<SplitSink<WebSocket, GlooMsg>>,
}

impl Connection {
    fn new(link: yew_agent::AgentLink<WS>) -> Self {
        log::info!("Opening websocket connection");

        let window = web_sys::window().expect("could not access window");
        let hostname = window
            .location()
            .hostname()
            .unwrap_or_else(|e| panic!("could not get hostname: {:?}", e));

        // TODO: make the port configurable somehow
        let ws = WebSocket::open(&format!("ws://{}:1337", hostname))
            .expect("only errors if url is bad?");
        let (mut tx, mut rx) = ws.split();

        let link2 = link.clone();
        let (close_rx, mut should_close) = oneshot::channel();
        let (one_tx, rx_end) = oneshot::channel();
        spawn_local(async move {
            loop {
                select! {
                    // TODO: this gives an error if this first branch is taken
                    // is it the fuse's fault?
                    // try tokio select instead?
                    _ = &mut should_close => break,
                    opt_msg = rx.next().fuse() => {
                        match opt_msg {
                            None => break,
                            Some(msg) =>
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
                    }
                }
            }

            link2.send_message(WSOutput::Conn(false));
            if one_tx.send(rx).is_err() {
                log::error!("rx_end failed to send back rx");
            }
            log::info!("Websocket rx_end closed");
        });

        let (ctx, mut crx) = mpsc::channel::<protocol::Message>(1000);
        let link2 = link.clone();
        let (one_tx, tx_end) = oneshot::channel();
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
            if one_tx.send(tx).is_err() {
                log::error!("tx_end failed to send back tx");
            }
            log::info!("Websocket tx_end closed");
        });

        Self {
            tx_end,
            rx_end,
            ctx,
            close_rx,
        }
    }

    fn close(mut self) {
        self.ctx.disconnect();
        self.close_rx.send(()).ok();

        spawn_local(async move {
            log::debug!("Waiting for both ends of connection");
            let (rx, tx) = join!(self.rx_end, self.tx_end);
            if rx.is_err() || tx.is_err() {
                log::error!("One of rx and tx did not receive their corresponding halve");
                return;
            }

            match rx.unwrap().reunite(tx.unwrap()) {
                Err(e) => log::error!("Failed to reunite both ends: {}", e),
                Ok(websocket) => {
                    match websocket.close(Some(1000), Some("Client wishes to close")) {
                        Err(e) => log::error!("Did not close successfully: {}", e),
                        Ok(()) => log::info!("Successfully closed connection"),
                    }
                }
            }
        });
    }

    fn sender(&mut self) -> &mut mpsc::Sender<protocol::Message> {
        &mut self.ctx
    }
}

impl Agent for WS {
    type Reach = yew_agent::Context<Self>;
    type Message = WSInternal;
    type Input = protocol::Message;
    type Output = WSOutput;

    fn create(link: yew_agent::AgentLink<Self>) -> Self {
        let window = web_sys::window().expect("could not access window");
        let document = window.document().expect("could not access document");

        let link2 = link.clone();
        let vischange = EventListener::new(&window, "visibilitychange", move |_| {
            link2.send_message(WSInternal::Visible(
                document.visibility_state() == web_sys::VisibilityState::Visible,
            ));
        });

        WS {
            link: link.clone(),
            clients: HashSet::new(),
            connection: Some(Connection::new(link)),
            connected: false,
            _vischange: vischange,
        }
    }

    fn destroy(&mut self) {
        if let Some(conn) = self.connection.take() {
            conn.close();
        }
    }

    fn update(&mut self, msg: Self::Message) {
        let distribute = match msg {
            WSInternal::Out(WSOutput::Conn(conn)) if conn == self.connected => None,
            WSInternal::Out(WSOutput::Conn(conn)) => {
                self.connected = conn;
                if conn {
                    log::info!("Connection established");
                } else {
                    log::info!("Connection lost");
                }
                Some(WSOutput::Conn(conn))
            }
            WSInternal::Visible(visible) => {
                log::debug!("Page visibility changed: visible={}", visible);
                if !visible {
                    if let Some(conn) = self.connection.take() {
                        conn.close();
                    }
                } else {
                    if self.connection.is_none() {
                        self.connection = Some(Connection::new(self.link.clone()));
                    } else {
                        log::warn!(
                            "Tried to create a new connection due to visibility \
                             changing, but a connection was already up"
                        );
                    }
                }
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
        if let Some(conn) = &mut self.connection {
            if let Err(e) = conn.sender().try_send(msg) {
                log::error!("Failed to send to WS: {}", e);
            }
        } else {
            log::warn!(
                "Tried to send '{:?}' but there is definitely no connection up",
                msg
            );
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
