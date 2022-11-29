use protocol::to_server::{mpvstart, playurlstart};

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::back_button::{BackButton, Type};
use crate::{websocket::websocket_send, WebSockStatus};

#[rustfmt::skip::macros(html)]
#[function_component(PlayUrl)]
pub fn playurl() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");

    let url = use_state(|| "".to_string());
    let url_change = {
        let url_setter = url.setter();

        Callback::from(move |ie: InputEvent| {
            let input = ie
                .target()
                .and_then(|target| target.dyn_into().ok())
                .map(|ele: HtmlInputElement| ele.value());

            match input {
                Some(inp) => url_setter.set(inp),
                None => log::error!("Could not get value from text input"),
            }
        })
    };

    let play_click = {
        let url2 = url.clone();
        Callback::from(move |_| {
            websocket_send(mpvstart::Url((*url2).clone()));
        })
    };

    html! {
        <article class={classes!("stacker")}>
            <BackButton button_type={Type::Back}
                        onclick={click_send!(playurlstart::Stop)} />
            <input type="url"
                   value={(*url).clone()}
                   class={classes!()}
                   oninput={url_change}
                   placeholder={"http://"}
                   disabled={active.is_disconnected()}
            />
            <button class={classes!()}
                    disabled={active.is_disconnected() || url.is_empty()}
                    onclick={play_click}>
                {"Play"}
            </button>
        </article>
    }
}
