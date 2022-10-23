use yew::prelude::*;

use crate::WebSockStatus;

#[rustfmt::skip::macros(html)]
#[function_component(Pending)]
pub fn pending() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    let inactive_class = active.is_disconnected().then_some("icon-error");
    let active_class = active.is_connected().then_some("spin");

    html! {
        <article class={classes!("center-page")}>
            <span class={classes!("icon", "icon-renew", inactive_class, "big", active_class)}></span>
        </article>
    }
}
