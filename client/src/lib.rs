// Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]

mod websocket;

use protocol::to_client::{seat::Seat, ToClient};
use wasm_bindgen::prelude::wasm_bindgen;
use websocket::{use_websocket, use_websocket_status};
use yew::prelude::*;

#[derive(PartialEq)]
pub enum Accepted {
    Pending,
    Accepted,
    Rejected,
}

#[function_component(App)]
fn app() -> Html {
    let accepted = use_state_eq(|| Accepted::Pending);
    let ws_ready = use_state_eq(|| false);
    let _ws_status = {
        let ws_ready2 = ws_ready.clone();
        use_websocket_status(move |b| ws_ready2.set(b))
    };
    let _ws = {
        let accepted2 = accepted.clone();
        use_websocket(move |m| match m.client_kind() {
            ToClient::Seat(Seat::Accept) => accepted2.set(Accepted::Accepted),
            ToClient::Seat(Seat::Reject) => accepted2.set(Accepted::Rejected),
        })
    };

    html! {<>
    <p>{if *ws_ready {"connected"} else {"disconnected"}}</p>
    <p>{match *accepted {
        Accepted::Pending => "pending",
        Accepted::Accepted => "accapted",
        Accepted::Rejected => "rejected",
    }}
    </p>
    </>}
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
