use protocol::to_server::{spotifyctrl, spotifystart};
use yew::prelude::*;

use crate::hooks::server::UseServer;

#[rustfmt::skip::macros(html)]
#[function_component(Spotify)]
pub fn spotify() -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    html! {
        <article class={classes!("stacker")}>
            <header class={classes!("center")}>
                <h1>{"Spotify"}</h1>
            </header>
            <button onclick={click_send!(server, spotifyctrl::Fullscreen)}
                    disabled={server.is_disconnected()}>
                {"Enter fullscreen"}
            </button>
            <button onclick={click_send!(server, spotifystart::Stop)}
                    class={classes!("error")}
                    disabled={server.is_disconnected()}>
                {"Close"}
            </button>
        </article>
    }
}
