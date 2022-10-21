use protocol::to_client::front::filesearch::SearchResult;

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::WebSockStatus;

#[rustfmt::skip::macros(html)]
#[function_component(Filesearch)]
pub fn filesearch() -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");

    let query = use_state(|| "".to_string());
    let query_change = {
        let query_setter = query.setter();
        Callback::from(move |ie: InputEvent| {
            let input = ie
                .target()
                .and_then(|target| target.dyn_into().ok())
                .map(|ele: HtmlInputElement| ele.value());

            match input {
                Some(inp) => query_setter.set(inp),
                None => log::error!("Could not get value from text input"),
            }
        })
    };

    let results = use_state(|| Vec::new());
    let results_html: Html = (*results)
        .iter()
        .map(|res: &SearchResult| html! {<div>{res.path.clone()}</div>})
        .collect();

    html! {
        <>
            <input type="text"
                   value={(*query).clone()}
                   oninput={query_change}
                   placeholder={"Search query"}
                   disabled={active.is_disconnected()}
            />
            {results_html}
        </>
    }
}
