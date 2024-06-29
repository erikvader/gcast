use futures::{
    channel::{mpsc, oneshot},
    select,
    stream::{Fuse, SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message as GlooMsg, WebSocketError};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::{
    functional::{hook, use_effect, use_mut_ref, use_state, use_state_eq},
    UseStateSetter,
};

#[derive(Clone)]
pub struct WS {
    sender: Sender,
    wish: UseStateSetter<Wish>,
    message: Option<Rc<Vec<u8>>>,
}

#[derive(Clone)]
pub struct Sender {
    sender: Option<mpsc::Sender<Vec<u8>>>,
}

impl Sender {
    pub fn send(&mut self, msg: Vec<u8>) -> Result<(), ()> {
        match &mut self.sender {
            None => Err(()),
            Some(sender) => sender.try_send(msg).map_err(|_| ()),
        }
    }

    fn new(sender: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    pub fn empty() -> Self {
        Self { sender: None }
    }
}

impl WS {
    /// `None` if not connected, else `Some(msg)` with the latest received message.
    pub fn message(&self) -> Option<Rc<Vec<u8>>> {
        self.message.clone()
    }

    pub fn is_connected(&self) -> bool {
        self.message().is_some()
    }

    pub fn open(&self) {
        self.wish.set(Wish::ToOpen);
    }

    pub fn close(&self) {
        self.wish.set(Wish::ToClose);
    }

    pub fn sender(&self) -> Sender {
        self.sender.clone()
    }
}

#[derive(PartialEq, Eq)]
enum State {
    Connecting,
    Connected,
    Closing,
    Closed,
}

#[derive(PartialEq, Eq)]
enum Wish {
    ToOpen,
    ToClose,
}

#[hook]
pub fn use_websocket(port: u16) -> WS {
    let state = use_state(|| State::Connecting);

    let stream = use_mut_ref(|| None::<Fuse<SplitStream<WebSocket>>>);
    let sink = use_mut_ref(|| None::<SplitSink<WebSocket, GlooMsg>>);
    let stream_close = use_mut_ref(|| None::<oneshot::Sender<()>>);
    let sink_close = use_mut_ref(|| None::<oneshot::Sender<()>>);
    let sink_ctrl = use_mut_ref(|| Sender::empty());

    let wish = use_state_eq(|| Wish::ToOpen);

    let message = use_state(|| None::<Rc<Vec<u8>>>);

    {
        let state = state.clone();
        let stream = stream.clone();
        let sink = sink.clone();
        let stream_close = stream_close.clone();
        let sink_close = sink_close.clone();
        let sink_ctrl = sink_ctrl.clone();
        let wish = wish.clone();
        let message = message.clone();
        use_effect(move || {
            match *state {
                State::Connecting => {
                    if *wish == Wish::ToClose {
                        state.set(State::Closed);
                        return;
                    }
                    state.set(State::Connected);
                    log::info!(
                        "Opening websocket connection to the same host on port {}",
                        port
                    );

                    {
                        let window = web_sys::window().expect("could not access window");
                        let hostname = window.location().hostname().unwrap_or_else(|e| {
                            panic!("could not get hostname: {:?}", e)
                        });

                        let ws = WebSocket::open(&format!("ws://{}:{}", hostname, port))
                            .expect("only errors if url is bad?");
                        let (tx, rx) = ws.split();
                        *stream.borrow_mut() = Some(rx.fuse());
                        *sink.borrow_mut() = Some(tx);
                    }

                    {
                        let state = state.clone();
                        let wish = wish.clone();
                        let (close, mut should_close) = oneshot::channel::<()>();
                        *stream_close.borrow_mut() = Some(close);
                        spawn_local(async move {
                            let mut stream = stream.borrow_mut();
                            loop {
                                select! {
                                    _ = &mut should_close => break,
                                    msg = stream.as_mut().unwrap().next() => {
                                        match msg {
                                            None => break,
                                            Some(msg) => {
                                                match msg {
                                                    Ok(GlooMsg::Bytes(msg)) => message.set(Some(Rc::new(msg))),
                                                    Ok(GlooMsg::Text(msg)) => log::warn!("Received a text message: {}", msg),
                                                    Err(WebSocketError::ConnectionClose(e)) => {
                                                        log::warn!("Websocket disconnected: {:?}", e);
                                                        break;
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to read: {}", e);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            wish.set(Wish::ToClose);
                            state.set(State::Closing);
                            log::debug!("Receiver future closed");
                        });
                    }

                    {
                        let (tx, mut sink_recv) = mpsc::channel(1024);
                        *sink_ctrl.borrow_mut() = Sender::new(tx);
                        let (close, mut should_close) = oneshot::channel::<()>();
                        *sink_close.borrow_mut() = Some(close);
                        spawn_local(async move {
                            let mut sink = sink.borrow_mut();
                            loop {
                                select! {
                                    _ = &mut should_close => break,
                                    tosend = sink_recv.next() => {
                                        let tosend = tosend.expect("there should always be at least one sender: sink_ctrl");
                                        match sink.as_mut().unwrap().send(GlooMsg::Bytes(tosend)).await {
                                            Ok(()) => {
                                                // NOTE: This branch is taken if websocket is already closed for
                                                // some reason.
                                                // 'WebSocket is already in CLOSING or CLOSED state'
                                            }
                                            Err(WebSocketError::ConnectionClose(e)) => {
                                                log::info!("Websocket disconnected: {:?}", e);
                                                break;
                                            }
                                            Err(e) => {
                                                log::error!("Failed to send: {}", e);
                                                break;
                                            },
                                        }
                                    }
                                }
                            }
                            wish.set(Wish::ToClose);
                            state.set(State::Closing);
                            log::debug!("Sender future closed");
                        });
                    }
                }
                State::Connected => {
                    if *wish == Wish::ToClose {
                        log::info!("Wish to close received");
                        state.set(State::Closing);
                    }
                }
                State::Closing => {
                    if let Some(closer) = stream_close.take() {
                        closer.send(()).ok();
                    }
                    if let Some(closer) = sink_close.take() {
                        closer.send(()).ok();
                    }

                    match (stream.try_borrow_mut(), sink.try_borrow_mut()) {
                        (Ok(mut one), Ok(mut other)) => {
                            state.set(State::Closed);
                            *sink_ctrl.borrow_mut() = Sender::empty();
                            message.set(None);

                            let one = one.take().unwrap().into_inner();
                            let other = other.take().unwrap();
                            match one.reunite(other) {
                                Err(e) => {
                                    log::error!("Failed to reunite both ends: {}", e)
                                }
                                Ok(websocket) => {
                                    match websocket
                                        .close(Some(1000), Some("Client wishes to close"))
                                    {
                                        Err(e) => log::error!(
                                            "Did not close successfully: {}",
                                            e
                                        ),
                                        Ok(()) => {
                                            log::info!("Successfully closed connection")
                                        }
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                State::Closed => {
                    if *wish == Wish::ToOpen {
                        log::info!("Wish to open received");
                        state.set(State::Connecting);
                    }
                }
            }
        });
    }

    let sink_ctrl = sink_ctrl.borrow_mut();
    WS {
        message: (*message).clone(),
        wish: wish.setter(),
        sender: sink_ctrl.clone(),
    }
}
