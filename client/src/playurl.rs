use super::UseServer;
use protocol::to_server::{mpvstart, playurlstart};

use wasm_bindgen::JsCast;
use web_sys::{window, HtmlDocument, HtmlInputElement};
use yew::prelude::*;

use crate::back_button::{BackButton, Type};

#[rustfmt::skip::macros(html)]
#[function_component(PlayUrl)]
pub fn playurl() -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    let input_ref = use_node_ref();

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

    let paste_click = {
        let input_ref = input_ref.clone();
        Callback::from(move |_| {
            focus(&input_ref);
            let doc: HtmlDocument = window()
                .expect("failed to get window")
                .document()
                .expect("failed to get document")
                .dyn_into()
                .expect("failed to cast into html document");

            match doc.exec_command("paste") {
                Ok(true) => (),
                Ok(false) => log::error!("Not allowed to paste"),
                Err(e) => log::error!("Failed to paste: {e:?}"),
            };
        })
    };

    use_effect_with(input_ref.clone(), |input_ref| focus(input_ref));

    html! {
        <article class={classes!("stacker")}>
            <BackButton button_type={Type::Back}
                        onclick={click_send!(server, playurlstart::Stop)} />
            <input type="url"
                   ref={input_ref}
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
            <button class={classes!()}
                    disabled={!url.is_empty()}
                    onclick={paste_click}>
                {"Paste"}
            </button>
        </article>
    }
}

fn focus(input_ref: &NodeRef) {
    let input = input_ref
        .cast::<HtmlInputElement>()
        .expect("ref not attached");

    if let Err(e) = input.focus() {
        log::error!("Failed to focus the input field: {e:?}");
    }
}
