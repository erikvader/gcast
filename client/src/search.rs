use protocol::{
    to_client::front::filesearch as prot,
    to_server::{fscontrol, fsstart, mpvstart},
};

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{websocket::websocket_send, WebSockStatus};

#[derive(Properties, PartialEq)]
pub struct FilesearchProps {
    pub front: prot::FileSearch,
}

#[rustfmt::skip::macros(html)]
#[function_component(Filesearch)]
pub fn filesearch(props: &FilesearchProps) -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    html! {
        <>
            <button onclick={click_send!(fsstart::Stop)} disabled={active.is_disconnected()}>{"Go back"}</button>
            {match &props.front {
                prot::FileSearch::Init(init) => html!{<Init front={init.clone()} />},
                prot::FileSearch::Refreshing(refr) => html!{<Refreshing front={refr.clone()} />},
                prot::FileSearch::Results(res) => html!(<Results front={res.clone()} />),
            }}
        </>
    }
}

#[derive(Properties, PartialEq)]
struct ResultsProps {
    front: prot::Results,
}

#[rustfmt::skip::macros(html)]
#[function_component(Results)]
fn results(props: &ResultsProps) -> Html {
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
                Some(inp) => {
                    query_setter.set(inp.clone());
                    websocket_send(fscontrol::Search(inp));
                }
                None => log::error!("Could not get value from text input"),
            }
        })
    };

    let results_html: Html = props
        .front
        .results
        .iter()
        .map(|res| html! {<SearchResult front={res.clone()} />})
        .collect();

    html! {
        <>
            <input type="text"
                   value={(*query).clone()}
                   oninput={query_change}
                   placeholder={"Search query"}
                   disabled={active.is_disconnected()}
            />
            <div>{format!("Query: '{}', valid: {}", props.front.query, props.front.query_valid)}</div>
            {results_html}
        </>
    }
}

#[derive(Properties, PartialEq)]
struct SearchResultProps {
    front: prot::SearchResult,
}

#[rustfmt::skip::macros(html)]
#[function_component(SearchResult)]
fn search_result(props: &SearchResultProps) -> Html {
    let contents: Html = searcher::stylize(
        &props.front.path,
        &props.front.indices,
        |on| html! {<span style="color: red">{on}</span>},
        |off| html! {off},
    );

    let on_click = {
        let root = props.front.root;
        let path = props.front.path.clone();
        Callback::from(move |_| {
            websocket_send(mpvstart::File {
                root,
                path: path.clone(),
            })
        })
    };

    // TODO: handle disconnection from server
    html! {
        <div onclick={on_click}>{contents}</div>
    }
}

#[derive(Properties, PartialEq)]
struct RefreshingProps {
    front: prot::Refreshing,
}

#[rustfmt::skip::macros(html)]
#[function_component(Refreshing)]
fn refreshing(props: &RefreshingProps) -> Html {
    html! {
        <>
            <div>{"Refreshing cache..."}</div>
            <div>{format!("Exploding: {}", props.front.exploding)}</div>
            <div>{format!("{}%", props.front.progress)}</div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct InitProps {
    front: prot::Init,
}

#[rustfmt::skip::macros(html)]
#[function_component(Init)]
fn init(props: &InitProps) -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");

    html! {
        <>
            <div>{cache_date(props.front.last_cache_date)}</div>
            <button disabled={active.is_disconnected()}
                    onclick={click_send!(fscontrol::RefreshCache)}>
                {"Refresh cache"}
            </button>
            <button disabled={active.is_disconnected() || props.front.last_cache_date.is_none()}
                    onclick={click_send!(fscontrol::Search("".to_string()))}>
                {"Search"}
            </button>
            // TODO: button for youtube links here?
        </>
    }
}

fn cache_date(time: Option<std::time::SystemTime>) -> String {
    match time {
        Some(st) => {
            let local: chrono::DateTime<chrono::Local> = st.into();
            format!("Cache last updated on: {}", local.format("%Y-%m-%d %H:%M"))
        }
        None => "There is no cache yet".to_string(),
    }
}
