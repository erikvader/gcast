use crate::WebSockStatus;
use protocol::to_server::spotifystart;
use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Spotify)]
pub fn spotify() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    html! {
        <article class={classes!("stacker")}>
            <header class={classes!("center")}>
                <h1>{"Spotify"}</h1>
            </header>
            <button onclick={click_send!(spotifystart::Stop)}
                    class={classes!("error")}
                    disabled={active.is_disconnected()}>
                {"Close"}
            </button>
        </article>
    }
}
