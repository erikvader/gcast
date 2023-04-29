use yew::prelude::*;

use crate::UseServer;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Type {
    Back,
    Exit,
}

#[derive(Properties, PartialEq)]
pub struct BackButtonProps {
    pub button_type: Type,
    pub onclick: Callback<MouseEvent>,
}

#[rustfmt::skip::macros(html)]
#[function_component(BackButton)]
pub fn backbutton(props: &BackButtonProps) -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    let (text, icon, class) = match props.button_type {
        Type::Back => ("Go back", "icon-back-arrow", None),
        Type::Exit => ("Exit", "icon-close", Some("error")),
    };

    html! {
        <button onclick={props.onclick.clone()}
                class={classes!("left", icon, "icon-right", "icon", class, "relative")}
                disabled={server.is_disconnected()}>
            <span class={classes!("vertical")}>{text}</span>
        </button>
    }
}
