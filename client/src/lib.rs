// Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]

macro_rules! click_send {
    ($send:expr) => {{
        use $crate::websocket::websocket_send;
        Callback::from(|_| websocket_send($send))
    }};
}

mod mpv;
mod nothing;
mod pending;
mod rejected;
mod search;
mod spotify;
mod websocket;

use mpv::Mpv;
use nothing::Nothing;
use pending::Pending;
use protocol::{
    to_client::{front::Front, seat::Seat, ToClient},
    to_server::sendstatus::SendStatus,
    ToMessage,
};
use rejected::Rejected;
use search::Filesearch;
use spotify::Spotify;
use wasm_bindgen::prelude::wasm_bindgen;

use websocket::{use_websocket, use_websocket_status, websocket_send};
use yew::prelude::*;

#[derive(PartialEq)]
enum Accepted {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum WebSockStatus {
    Connected,
    Disconnected,
}

impl From<bool> for WebSockStatus {
    fn from(b: bool) -> Self {
        if b {
            Self::Connected
        } else {
            Self::Disconnected
        }
    }
}

impl WebSockStatus {
    fn is_connected(self) -> bool {
        matches!(self, WebSockStatus::Connected)
    }
    fn is_disconnected(self) -> bool {
        !self.is_connected()
    }
}

#[rustfmt::skip::macros(html)]
#[function_component(App)]
fn app() -> Html {
    let accepted = use_state_eq(|| Accepted::Pending);
    let ws_ready = use_state_eq(|| WebSockStatus::Disconnected);
    let front = use_state_eq::<Option<(u64, Front)>, _>(|| None);
    let _ws_status = {
        let ws_ready_setter = ws_ready.setter();
        use_websocket_status(move |b| ws_ready_setter.set(b.into()))
    };
    let _ws = {
        let accepted_setter = accepted.setter();
        let front_clone = front.clone();
        use_websocket(move |m| match m.borrow_to_client() {
            ToClient::Seat(Seat::Accept) => {
                accepted_setter.set(Accepted::Accepted);
                websocket_send(SendStatus.to_message());
            }
            ToClient::Seat(Seat::Reject) => {
                accepted_setter.set(Accepted::Rejected);
            }
            ToClient::Front(front) => {
                let f = (*front_clone).as_ref();
                if f.is_none() || m.is_newer_than(f.unwrap().0) {
                    front_clone.set(Some((m.id(), front.clone())));
                }
            }
            ToClient::Notification(_) => todo!(),
        })
    };

    html! {
        <ContextProvider<WebSockStatus> context={*ws_ready}>
            {match (&*accepted, &*front) {
                (Accepted::Pending, _) | (Accepted::Accepted, None) => html! {<Pending />},
                (Accepted::Rejected, _) => html! {<Rejected />},
                (Accepted::Accepted, Some((_, Front::None))) => html! {<Nothing />},
                (Accepted::Accepted, Some((_, Front::Spotify))) => html! {<Spotify />},
                (Accepted::Accepted, Some((_, Front::Mpv(mpv)))) => html! {<Mpv front={mpv.clone()} />},
                (Accepted::Accepted, Some((_, Front::FileSearch(fs)))) => html! {<Filesearch front={fs.clone()} />},
            }}
        </ContextProvider<WebSockStatus>>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
