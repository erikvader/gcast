use crate::{websocket::websocket_send, WebSockStatus};
use protocol::to_server::{fsstart, spotifystart};
use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Nothing)]
pub fn nothing() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    let to_spotify = Callback::from(|_| websocket_send(spotifystart::Start));
    let to_filesearch = Callback::from(|_| websocket_send(fsstart::Start));
    html! {
        <>
            <button onclick={to_spotify}
                    disabled={active.is_disconnected()}>
                {"Spotify"}
            </button>
            <button onclick={to_filesearch}
                    disabled={active.is_disconnected()}>
                {"File Search"}
            </button>
        </>
    }
}
