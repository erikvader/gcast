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
    // TODO: style this
    html! {
        <article class={classes!("stacker")}>
            <header class={classes!()}>
                <h1>{&props.front.header}</h1>
            </header>
            <p>{&props.front.body}</p>
            <button onclick={click_send!(Close)}
                    class={classes!()}
                    disabled={active.is_disconnected()}>
                {"Ok"}
            </button>
        </article>
    }
}
