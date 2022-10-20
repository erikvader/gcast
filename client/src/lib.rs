// Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]

mod websocket;

use protocol::{
    to_client::{
        front::{filesearch::SearchResult, Front},
        seat::Seat,
        ToClient,
    },
    to_server::{fsstart, sendstatus::SendStatus, spotifystart},
    ToMessage,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast};
use web_sys::HtmlInputElement;
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
        <ContextProvider<bool> context={*ws_ready}> // TODO: don't use a simple bool, use custom enum
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

// TODO: put in own module
#[rustfmt::skip::macros(html)]
#[function_component(Filesearch)]
fn filesearch() -> Html {
    let active = use_context::<bool>().expect("no active context found");

    let query = use_state(|| "".to_string());
    let query_change = {
        let query_setter = query.setter();
        Callback::from(move |ie: InputEvent| {
            let input = ie
                .target()
                .and_then(|target| target.dyn_into().ok())
                .map(|ele: HtmlInputElement| ele.value());

            match input {
                Some(inp) => query_setter.set(inp),
                None => log::error!("Could not get value from text input"),
            }
        })
    };

    let results = use_state(|| Vec::new());
    let results_html: Html = (*results)
        .iter()
        .map(|res: &SearchResult| html! {<div>{res.path.clone()}</div>})
        .collect();

    html! {
        <>
            <input type="text"
                   value={(*query).clone()}
                   oninput={query_change}
                   placeholder={"Search query"}
                   disabled={!active}
            />
            {results_html}
        </>
    }
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
    let to_filesearch = send_callback!(ws, fsstart::Start);
    html! {
        <>
            <button onclick={to_spotify} disabled={!active}>{"Spotify"}</button>
            <button onclick={to_filesearch} disabled={!active}>{"File Search"}</button>
        </>
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
