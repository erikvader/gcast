use super::UseServer;
use protocol::to_server::{mpvstart, playurlstart};

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::back_button::{BackButton, Type};

#[rustfmt::skip::macros(html)]
#[function_component(PlayUrl)]
pub fn playurl() -> Html {
    let server = use_context::<UseServer>().expect("no server context found");

    // TODO: initialize with clipboard contents if it as an URL. Probably use a lib for
    // this since checking if a string is a URL is done in multiple places
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
        let url = url.clone();
        click_send!(server, mpvstart::url::Url((*url).clone()))
    };

    html! {
        <article class={classes!("stacker")}>
            <BackButton button_type={Type::Back}
                        onclick={click_send!(server, playurlstart::Stop)} />
            <input type="url"
                   value={(*url).clone()}
                   class={classes!()}
                   oninput={url_change}
                   placeholder={"http://"}
                   disabled={server.is_disconnected()}
            />
            <button class={classes!()}
                    disabled={server.is_disconnected() || url.is_empty()}
                    onclick={play_click}>
                {"Play"}
            </button>
        </article>
    }
}
