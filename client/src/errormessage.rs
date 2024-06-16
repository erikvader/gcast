use crate::UseServer;
use protocol::to_client::front::errormsg as prot;
use protocol::to_server::errormsgctrl::Close;
use yew::prelude::*;

#[derive(Properties, PartialEq, Eq)]
pub struct ErrorProps {
    pub front: prot::ErrorMsg,
}

#[rustfmt::skip::macros(html)]
#[function_component(ErrorMessage)]
pub fn errormessage(props: &ErrorProps) -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    html! {
        <article class={classes!("stacker")}>
            <h2 class={classes!("error", "pad", "white-text")}>
                {"Something Exceptional Happened"}
            </h2>
            <header class={classes!("pad")}>
                <h2>{&props.front.header}</h2>
            </header>
            <p class={classes!("pad", "pre-wrap")}>{&props.front.body}</p>
            <button onclick={click_send!(server, Close)}
                    class={classes!()}
                    disabled={server.is_disconnected()}>
                {"Ok"}
            </button>
        </article>
    }
}
