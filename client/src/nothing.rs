use crate::confirm_button::ConfirmButton;
use crate::WebSockStatus;
use protocol::to_server::{fsstart, powerctrl, spotifystart};
use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Nothing)]
pub fn nothing() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    html! {
        <article class={classes!("stacker")}>
            <header class={classes!("center")}>
                <h1>
                    <span class={classes!("leckerli", "embellishment")}>{"g"}</span>{"cast"}
                </h1>
            </header>
            <button onclick={click_send!(spotifystart::Start)}
                    class={classes!("icon", "icon-radio", "icon-hspace")}
                    disabled={active.is_disconnected()}>
                {"Spotify"}
            </button>
            <button onclick={click_send!(fsstart::Start)}
                    class={classes!("icon", "icon-camera", "icon-hspace")}
                    disabled={active.is_disconnected()}>
                {"Play video"}
            </button>
            <ConfirmButton onclick={click_send!(powerctrl::Poweroff)}
                           unarmed_classes={classes!("icon", "icon-power", "icon-hspace")}
                           unarmed_text={"Power off"}
                           armed_classes={classes!("error", "icon", "icon-front-hand", "icon-hspace")} />
        </article>
    }
}
