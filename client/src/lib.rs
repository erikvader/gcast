// NOTE: Remove annoying warning from wasm_bindgen
#![allow(non_snake_case, non_upper_case_globals)]
// NOTE: clippy doesn't like yew html macro
#![allow(clippy::let_unit_value)]

macro_rules! click_send {
    ($server:expr, $send:expr) => {{
        let sender = $server.sender();
        Callback::from(move |_| sender.send($send))
    }};
    ($server:expr, $arg:ident -> $send:expr) => {{
        let sender = $server.sender();
        Callback::from(move |$arg| sender.send($send))
    }};
}

mod back_button;
mod confirm_button;
mod debounce;
mod debug;
mod errormessage;
mod hooks;
mod mpv;
mod nothing;
mod pending;
mod playurl;
mod progressbar;
mod rejected;
mod search;
mod spotify;

use errormessage::ErrorMessage;
use hooks::server::UseServer;
use mpv::Mpv;
use nothing::Nothing;
use pending::Pending;
use playurl::PlayUrl;
use protocol::to_client::front::Front;
use rejected::Rejected;
use search::Filesearch;
use spotify::Spotify;
use wasm_bindgen::prelude::wasm_bindgen;

use yew::prelude::*;

use crate::hooks::server::{use_server, use_server_debug, Accepted};

#[derive(PartialEq, Properties)]
struct AppProps {
    server: UseServer,
}

#[rustfmt::skip::macros(html)]
#[function_component(App)]
fn app(props: &AppProps) -> Html {
    let server = &props.server;
    html! {
        <ContextProvider<UseServer> context={server.clone()}>
            <div class={classes!("width-limiter")}>
                {match (server.accepted(), server.front()) {
                    (Accepted::Pending, _) => html! {<Pending />},
                    (Accepted::Rejected, _) => html! {<Rejected />},
                    (Accepted::Accepted, Front::None) => html! {<Nothing />},
                    (Accepted::Accepted, Front::Spotify) => html! {<Spotify />},
                    (Accepted::Accepted, Front::Mpv(mpv)) => html! {<Mpv front={mpv.clone()} />},
                    (Accepted::Accepted, Front::FileSearch(fs)) => html! {<Filesearch front={fs.clone()} />},
                    (Accepted::Accepted, Front::PlayUrl) => html! {<PlayUrl />},
                    (Accepted::Accepted, Front::ErrorMsg(em)) => html! {<ErrorMessage front={em.clone()} />},
                }}
            </div>
        </ContextProvider<UseServer>>
    }
}

#[rustfmt::skip::macros(html)]
#[function_component(LiveApp)]
fn live_app() -> Html {
    let server = use_server();
    html! {
        <App server={server} />
    }
}

#[derive(PartialEq, Properties)]
struct DebugAppProps {
    debug: debug::Debug,
}

#[rustfmt::skip::macros(html)]
#[function_component(DebugApp)]
fn debug_app(props: &DebugAppProps) -> Html {
    let server = use_server_debug(&props.debug);
    html! {
        <App server={server} />
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    let logger_level = if cfg!(debug_assertions) {
        log::Level::Trace
    } else {
        log::Level::Debug
    };
    wasm_logger::init(wasm_logger::Config::new(logger_level));

    match debug::debug() {
        Ok(debug) => {
            log::info!("Entering the debug app");
            log::info!("URL input: {debug:#?}");
            log::info!(
                "Connected={}, Accepted={:?}",
                debug.is_connected(),
                debug.accepted()
            );
            log::info!("Front: {:#?}", debug.front());
            yew::Renderer::<DebugApp>::with_props(DebugAppProps { debug }).render();
        }
        Err(reason) => {
            log::info!("Entering the live app and not debug because: '{reason}'");
            yew::Renderer::<LiveApp>::new().render();
        }
    }
}
