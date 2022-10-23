use yew::prelude::*;

#[rustfmt::skip::macros(html)]
#[function_component(Rejected)]
pub fn rejected() -> Html {
    html! {
        <article class={classes!("center-page")}>
            <span class={classes!("icon", "icon-front-hand", "big")}></span>
        </article>
    }
}
