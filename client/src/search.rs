use std::ops::Range;

use protocol::{
    to_client::front::filesearch as prot,
    to_server::{fscontrol, fsstart, mpvstart},
};

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::back_button::{BackButton, Type};
use crate::progressbar::Progressbar;
use crate::{websocket::websocket_send, WebSockStatus};

const COLORS: &[&str] = &[
    "dracula-pink",
    "dracula-purple",
    "dracula-yellow",
    "dracula-orange",
    "dracula-cyan",
    "dracula-green",
    "dracula-red",
];

#[derive(Properties, PartialEq, Eq)]
pub struct FilesearchProps {
    pub front: prot::FileSearch,
}

#[rustfmt::skip::macros(html)]
#[function_component(Filesearch)]
pub fn filesearch(props: &FilesearchProps) -> Html {
    html! {
        <article class={classes!("stacker")}>
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

    let query = use_state(|| props.front.query.to_string());
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
    html! {
        <>
            <BackButton button_type={Type::Back}
                        onclick={click_send!(fscontrol::BackToTheBeginning)} />
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
    let dir = search_result_substr(
        &props.front.path,
        &props.front.indices,
        0..props.front.basename,
    );

    let base = search_result_substr(
        &props.front.path,
        &props.front.indices,
        props.front.basename..props.front.path.len(),
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

    let color_class = COLORS.get(props.front.root).copied().unwrap_or_else(|| {
        log::warn!("Too few colors for the amount of roots in search detail");
        "dracula-black"
    });

    // TODO: handle disconnection from server. Remove all results?
    html! {
        <div class={classes!("search-res")} onclick={on_click}>
            <span class={classes!("search-detail", color_class)}></span>
            <span class={classes!("search-content")}>
                <span class={classes!("kinda-small", "italic")}>{dir}</span>
                <br />
                <span>{base}</span>
            </span>
        </div>
    }
}

fn search_result_substr(path: &str, indices: &[usize], char_range: Range<usize>) -> Html {
    let substr: String = path
        .chars()
        .enumerate()
        .filter(|(i, _)| char_range.contains(i))
        .map(|(_, c)| c)
        .collect();

    let subindices: Vec<_> = indices
        .iter()
        .filter(|i| char_range.contains(i))
        .map(|i| i - char_range.start)
        .collect();

    searcher::stylize(
        &substr,
        &subindices,
        |on| html! {<span class={classes!("search-hl")}>{on}</span>},
        |off| html! {off},
    )
}

#[derive(Properties, PartialEq)]
struct RefreshingProps {
    front: prot::Refreshing,
}

#[rustfmt::skip::macros(html)]
#[function_component(Refreshing)]
fn refreshing(props: &RefreshingProps) -> Html {
    let done_dirs = props.front.done_dirs;
    let total_dirs = props.front.total_dirs;
    let dirs_progress = if total_dirs == 0 {
        0.0
    } else {
        100.0 * (done_dirs as f64) / (total_dirs as f64)
    };
    let roots = props.front.roots.iter().map(|rootinfo| {
        use prot::RootStatus::*;
        let class = match rootinfo.status {
            Pending => "root-pending",
            Loading => "root-loading",
            Error => "root-error",
            Done => "root-done",
        };
        html! {<div class={classes!(class)}>{rootinfo.path.to_string()}</div>}
    });

    html! {
        <>
            if props.front.is_done {
                <BackButton button_type={Type::Back}
                            onclick={click_send!(fscontrol::BackToTheBeginning)} />
            } else {
                <BackButton button_type={Type::Exit}
                            onclick={click_send!(fsstart::Stop)} />
            }
            <article class={classes!("stacker", "pad")}>
                <header>
                    <h3>{"Refreshing cache..."}</h3>
                </header>
                <div>{format!("Number of errors: {}", props.front.num_errors)}</div>
                <h4>{"Roots"}</h4>
                {for roots}
                <div class={classes!("pad")} />
                <Progressbar progress={dirs_progress}
                             text={format!("{}/{}", done_dirs, total_dirs)}
                             text_class={classes!("progressbar-text")}
                             outer_class={classes!("progressbar-outer")}
                             inner_class={classes!("progressbar-inner")}/>
            </article>
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
            <BackButton button_type={Type::Exit}
                        onclick={click_send!(fsstart::Stop)} />
            <div class={classes!("pad")}>{cache_date(props.front.last_cache_date)}</div>
            <button disabled={active.is_disconnected()}
                    class={classes!("icon-refresh", "icon", "icon-hspace")}
                    onclick={click_send!(fscontrol::RefreshCache)}>
                {"Refresh cache"}
            </button>
            <button disabled={active.is_disconnected() || props.front.last_cache_date.is_none()}
                    class={classes!("icon-search", "icon", "icon-hspace")}
                    onclick={click_send!(fscontrol::Search("".to_string()))}>
                {"Search"}
            </button>
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
