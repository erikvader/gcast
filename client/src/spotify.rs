use crate::{websocket::websocket_send, WebSockStatus};
use protocol::to_server::spotifystart;
use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Spotify)]
pub fn spotify() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    let to_nothing = Callback::from(|_| websocket_send(spotifystart::Stop));
    html! {
        <button onclick={to_nothing} disabled={active.is_disconnected()}>{"Close"}</button>
    }
}
