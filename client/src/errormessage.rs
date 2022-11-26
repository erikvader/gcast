use crate::WebSockStatus;
use protocol::to_client::front::errormsg as prot;
use protocol::to_server::errormsgctrl::Close;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorProps {
    pub front: prot::ErrorMsg,
}

#[rustfmt::skip::macros(html)]
#[function_component(ErrorMessage)]
pub fn errormessage(props: &ErrorProps) -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    html! {
        <article class={classes!("stacker")}>
            <h2 class={classes!("error", "pad", "white-text")}>
                {"Something Exceptional Happened"}
            </h2>
            <header class={classes!("pad")}>
                <h2>{&props.front.header}</h2>
            </header>
            <p class={classes!("pad")}>{&props.front.body}</p>
            <button onclick={click_send!(Close)}
                    class={classes!()}
                    disabled={active.is_disconnected()}>
                {"Ok"}
            </button>
        </article>
    }
}
