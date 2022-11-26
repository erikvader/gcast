use yew::prelude::*;

use crate::WebSockStatus;

#[derive(Properties, PartialEq)]
pub struct ConfirmButtonProps {
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub armed_classes: Classes,
    #[prop_or_default]
    pub unarmed_classes: Classes,
    #[prop_or("Are you sure?".to_string())]
    pub armed_text: String,
    pub unarmed_text: String,
}

#[rustfmt::skip::macros(html)]
#[function_component(ConfirmButton)]
pub fn confirmbutton(props: &ConfirmButtonProps) -> Html {
    let armed = use_state_eq(|| false);
    let active = use_context::<WebSockStatus>().expect("no active context found");

    let onclick = {
        let armed2 = armed.clone();
        let onclick2 = props.onclick.clone();
        Callback::from(move |e| {
            if *armed2 {
                onclick2.emit(e);
            } else {
                armed2.set(true);
            }
        })
    };

    html! {
        <button onclick={onclick}
                class={if *armed {props.armed_classes.clone()} else {props.unarmed_classes.clone()}}
                disabled={active.is_disconnected()}>
            if !*armed {
                {props.unarmed_text.clone()}
            } else {
                {props.armed_text.clone()}
            }
        </button>
    }
}
