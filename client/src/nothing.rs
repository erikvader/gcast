use crate::{websocket::websocket_send, WebSockStatus};
use protocol::to_server::{fsstart, spotifystart};
use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Nothing)]
pub fn nothing() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    html! {
        <>
            <button onclick={click_send!(spotifystart::Start)}
                    disabled={active.is_disconnected()}>
                {"Spotify"}
            </button>
            <button onclick={click_send!(fsstart::Start)}
                    disabled={active.is_disconnected()}>
                {"Play video"}
            </button>
        </>
    }
}
