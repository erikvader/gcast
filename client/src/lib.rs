// Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]

use futures::{
    channel::mpsc,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use gloo_net::websocket::{futures::WebSocket, Message, State, WebSocketError};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

struct WS {
    tx: mpsc::Sender<String>,
}

impl WS {
    pub fn new(ws_ready: UseStateHandle<bool>) -> Self {
        log::info!("Opening websocket connection");
        let ws =
            WebSocket::open("ws://localhost:1337").expect("only errors if url is bad?");

        let (mut tx, mut rx) = ws.split();
        let ws_ready2 = ws_ready.clone();
        spawn_local(async move {
            while let Some(msg) = rx.next().await {
                match msg {
                    Ok(m) => {
                        log::info!("Received: {:?}", m);
                        ws_ready2.set(true);
                    }
                    Err(WebSocketError::ConnectionClose(e)) => {
                        log::warn!("websocket disconnected: {:?}", e);
                        ws_ready2.set(false);
                    }
                    Err(e) => {
                        log::error!("failed to read: {}", e);
                    }
                }
            }
            ws_ready2.set(false);
            log::info!("websocket closed");
        });

        let (ctx, mut crx) = mpsc::channel::<String>(1000);
        spawn_local(async move {
            while let Some(msg) = crx.next().await {
                match tx.send(Message::Text(msg)).await {
                    Ok(()) => ws_ready.set(true),
                    Err(WebSocketError::ConnectionClose(e)) => {
                        log::info!("websocket disconnected: {:?}", e);
                        ws_ready.set(false);
                    }
                    Err(e) => log::error!("failed to send: {}", e),
                }
            }
            ws_ready.set(false);
            log::error!("WS ctx dropped");
        });

        WS { tx: ctx }
    }

    pub fn send(&mut self, s: String) {
        if let Err(e) = self.tx.try_send(s) {
            log::error!("failed to send to WS: {}", e);
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let ws_ready = use_state_eq(|| false);
    let ws_tx = {
        let ws_ready = ws_ready.clone();
        use_mut_ref(move || WS::new(ws_ready))
    };
    {
        let ws_tx = ws_tx.clone();
        use_effect_with_deps(
            move |_| {
                ws_tx.borrow_mut().send("hej".to_string());
                || ()
            },
            (),
        );
    }

    html! {<p>{if *ws_ready {"connected"} else {"disconnected"}}</p>}
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
