use protocol::{
    to_client::front::filesearch as prot,
    to_server::{fscontrol, fsstart, mpvstart},
};

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::progressbar::Progressbar;
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
        <article class={classes!("stacker")}>
            <button onclick={click_send!(fsstart::Stop)}
                    class={classes!("left", "icon-back-arrow", "icon-right", "icon")}
                    disabled={active.is_disconnected()}>
                {"Go back"}
            </button>
            {match &props.front {
                prot::FileSearch::Init(init) => html!{<Init front={init.clone()} />},
                prot::FileSearch::Refreshing(refr) => html!{<Refreshing front={refr.clone()} />},
                prot::FileSearch::Results(res) => html!(<Results front={res.clone()} />),
            }}
        </article>
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

    // TODO: somehow reset the query to the one from the server on mount
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

    let invalid_class = (!props.front.query_valid).then_some("invalid");

    // TODO: show a loading icon when props.front.query != *query
    // TODO: handle scrolling
    html! {
        <>
            <input type="text"
                   value={(*query).clone()}
                   class={classes!(invalid_class)}
                   oninput={query_change}
                   placeholder={"Search query"}
                   autocapitalize={"none"}
                   disabled={active.is_disconnected()}
            />
            <div class={classes!("rows")}>
                {results_html}
            </div>
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
        |on| html! {<span class={classes!("search-hl")}>{on}</span>},
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

    // TODO: handle disconnection from server. Remove all results?
    html! {
        <div class={classes!("mono", "search-res")} onclick={on_click}>{contents}</div>
    }
}

#[derive(Properties, PartialEq)]
struct RefreshingProps {
    front: prot::Refreshing,
}

#[rustfmt::skip::macros(html)]
#[function_component(Refreshing)]
fn refreshing(props: &RefreshingProps) -> Html {
    let (upper, lower) = match props.front {
        prot::Refreshing {
            progress,
            exploding: true,
        } => (progress, 0),
        prot::Refreshing {
            progress,
            exploding: false,
        } => (100, progress),
    };

    html! {
        <article class={classes!("stacker", "pad")}>
            <header>
                <h3>{"Refreshing cache..."}</h3>
            </header>
            <Progressbar progress={upper}
                         outer_class={classes!("progressbar-outer")}
                         inner_class={classes!("progressbar-inner")}/>
            <div class={classes!("pad")} />
            <Progressbar progress={lower}
                         outer_class={classes!("progressbar-outer")}
                         inner_class={classes!("progressbar-inner")}/>
        </article>
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
            <div class={classes!("pad")}>{cache_date(props.front.last_cache_date)}</div>
            <button disabled={active.is_disconnected()}
                    class={classes!("icon-refresh", "icon")}
                    onclick={click_send!(fscontrol::RefreshCache)}>
                {"Refresh cache"}
            </button>
            <button disabled={active.is_disconnected() || props.front.last_cache_date.is_none()}
                    class={classes!("icon-search", "icon")}
                    onclick={click_send!(fscontrol::Search("".to_string()))}>
                {"Search"}
            </button>
            // TODO: button for youtube links here?
        </>
    }
}

#[rustfmt::skip::macros(html)]
fn cache_date(time: Option<std::time::SystemTime>) -> Html {
    match time {
        Some(st) => {
            let local: chrono::DateTime<chrono::Local> = st.into();
            html! {
                <>
                    <span class={classes!("bold")}>{"Cache last updated on: "}</span>
                    <span>{local.format("%Y-%m-%d %H:%M")}</span>
                </>
            }
        }
        None => html! {<span class={classes!("bold")}>{"There is no cache yet"}</span>},
    }
}
