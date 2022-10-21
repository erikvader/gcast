use protocol::{to_server::spotifystart, ToMessage};

use yew::prelude::*;

use crate::websocket::use_websocket;

#[rustfmt::skip::macros(html)]
#[function_component(Spotify)]
pub fn spotify() -> Html {
    let ws = use_websocket(|_| {});
    let active = use_context::<bool>().expect("no active context found");
    let to_nothing = send_callback!(ws, spotifystart::Stop);
    html! {
        <button onclick={to_nothing} disabled={!active}>{"Close"}</button>
    }
}
