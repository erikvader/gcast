use yew::prelude::*;

use crate::UseServer;

#[rustfmt::skip::macros(html)]
#[function_component(Pending)]
pub fn pending() -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    let inactive_class = server.is_disconnected().then_some("icon-error");
    let active_class = server.is_connected().then_some("spin");

    html! {
        <article class={classes!("center-page")}>
            <span class={classes!("icon", "icon-renew", inactive_class, "big", active_class)}></span>
        </article>
    }
}
