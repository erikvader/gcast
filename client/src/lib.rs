// Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]

mod websocket;

use protocol::{
    to_client::{seat::Seat, ToClient},
    to_server::sendstatus::SendStatus,
    ToMessage,
};
use wasm_bindgen::prelude::wasm_bindgen;
use websocket::{use_websocket, use_websocket_status};
use yew::{prelude::*, virtual_dom::AttrValue};

#[derive(PartialEq)]
pub enum Accepted {
    Pending,
    Accepted,
    Rejected,
}

#[rustfmt::skip::macros(html)]
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

    let should_be_active = *accepted == Accepted::Accepted && *ws_ready;
    html! {
        <ContextProvider<bool> context={should_be_active}>
            <p>{if *ws_ready {"connected"} else {"disconnected"}}</p>
            <p>{match *accepted {
                Accepted::Pending => "pending",
                Accepted::Accepted => "accepted",
                Accepted::Rejected => "rejected",}}
            </p>
            <Bewton text={"klicka här"} />
        </ContextProvider<bool>>
    }
}

#[derive(Properties, PartialEq)]
struct BewtonProps {
    text: AttrValue,
}

#[rustfmt::skip::macros(html)]
#[function_component(Bewton)]
fn bewton(props: &BewtonProps) -> Html {
    let ws = use_websocket(|_| {});
    let active = use_context::<bool>().expect("no active context found");
    let onclick = Callback::from(move |_| ws.send(SendStatus.to_message()));
    html! {
        <button onclick={onclick} disabled={!active}>{&props.text}</button>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
