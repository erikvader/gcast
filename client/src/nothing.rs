use crate::confirm_button::ConfirmButton;
use crate::UseServer;
use protocol::to_server::{fsstart, playurlstart, powerctrl, spotifystart};
use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Nothing)]
pub fn nothing() -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    html! {
        <article class={classes!("stacker")}>
            <header class={classes!("center")}>
                <h1>
                    <span class={classes!("leckerli", "embellishment")}>{"g"}</span>{"cast"}
                </h1>
            </header>
            <button onclick={click_send!(server, spotifystart::Start)}
                    class={classes!("icon", "icon-radio", "icon-hspace")}
                    disabled={server.is_disconnected()}>
                {"Spotify"}
            </button>
            <button onclick={click_send!(server, fsstart::Start)}
                    class={classes!("icon", "icon-camera", "icon-hspace")}
                    disabled={server.is_disconnected()}>
                {"Play video"}
            </button>
            <button onclick={click_send!(server, playurlstart::Start)}
                    class={classes!("icon", "icon-link", "icon-hspace")}
                    disabled={server.is_disconnected()}>
                {"Play video URL"}
            </button>
            <ConfirmButton onclick={click_send!(server, powerctrl::Poweroff)}
                           unarmed_classes={classes!("icon", "icon-power", "icon-hspace")}
                           unarmed_text={"Power off"}
                           armed_classes={classes!("error", "icon", "icon-front-hand", "icon-hspace")} />
        </article>
    }
}
