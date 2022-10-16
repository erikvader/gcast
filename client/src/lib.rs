// Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]

mod websocket;

use protocol::{
    to_client::{front::Front, seat::Seat, ToClient},
    to_server::{sendstatus::SendStatus, spotifystart},
    ToMessage,
};
use wasm_bindgen::prelude::wasm_bindgen;
use websocket::{use_websocket, use_websocket_send, use_websocket_status};
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
    let front = use_state_eq(|| None);
    let _ws_status = {
        let ws_ready_setter = ws_ready.setter();
        use_websocket_status(move |b| ws_ready_setter.set(b))
    };
    let _ws = {
        let accepted_setter = accepted.setter();
        let front_setter = front.setter();
        use_websocket_send(move |m| match m.borrow_to_client() {
            ToClient::Seat(Seat::Accept) => {
                accepted_setter.set(Accepted::Accepted);
                Some(SendStatus.to_message())
            }
            ToClient::Seat(Seat::Reject) => {
                accepted_setter.set(Accepted::Rejected);
                None
            }
            ToClient::Front(front) => {
                front_setter.set(Some(front.clone()));
                None
            }
            ToClient::Notification(_) => todo!(),
        })
    };

    html! {
        <ContextProvider<bool> context={*ws_ready}>
            {match (&*accepted, &*front) {
                (Accepted::Pending, _) | (Accepted::Accepted, None) => html! {<Pending />},
                (Accepted::Rejected, _) => html! {<Rejected />},
                (Accepted::Accepted, Some(Front::None)) => html! {<Nothing />},
                (Accepted::Accepted, Some(Front::Spotify)) => html! {<Spotify />},
                (Accepted::Accepted, Some(Front::Mpv(mpv))) => html! {<Mpv />},
                (Accepted::Accepted, Some(Front::FileSearch(fs))) => html! {<Filesearch />},
            }}
        </ContextProvider<bool>>
    }
}

#[rustfmt::skip::macros(html)]
#[function_component(Pending)]
fn pending() -> Html {
    html! {"pending"}
}

#[rustfmt::skip::macros(html)]
#[function_component(Filesearch)]
fn filesearch() -> Html {
    html! {"filesearch"}
}

#[rustfmt::skip::macros(html)]
#[function_component(Mpv)]
fn mpv() -> Html {
    html! {"mpv"}
}

macro_rules! send_callback {
    ($ws:ident, $send:expr) => {{
        let ws2 = $ws.clone();
        Callback::from(move |_| ws2.send($send.to_message()))
    }};
}

#[rustfmt::skip::macros(html)]
#[function_component(Nothing)]
fn nothing() -> Html {
    let ws = use_websocket(|_| {});
    let active = use_context::<bool>().expect("no active context found");
    let to_spotify = send_callback!(ws, spotifystart::Start);
    html! {
        <button onclick={to_spotify} disabled={!active}>{"Spotify"}</button>
    }
}

#[rustfmt::skip::macros(html)]
#[function_component(Spotify)]
fn spotify() -> Html {
    let ws = use_websocket(|_| {});
    let active = use_context::<bool>().expect("no active context found");
    let to_nothing = send_callback!(ws, spotifystart::Stop);
    html! {
        <button onclick={to_nothing} disabled={!active}>{"Close"}</button>
    }
}

#[rustfmt::skip::macros(html)]
#[function_component(Rejected)]
fn rejected() -> Html {
    html! {"rejected"}
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
