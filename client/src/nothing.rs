use protocol::{
    to_server::{fsstart, spotifystart},
    ToMessage,
};

use yew::prelude::*;

use crate::websocket::use_websocket;

#[rustfmt::skip::macros(html)]
#[function_component(Nothing)]
pub fn nothing() -> Html {
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
