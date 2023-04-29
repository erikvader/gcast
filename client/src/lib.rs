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

use crate::hooks::server::{use_server, Accepted};

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

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<LiveApp>::new().render();

    // use protocol::to_client::front::filesearch as fs;
    // use protocol::to_client::front::mpv as m;
    // yew::Renderer::<App>::with_props(AppProps {
    //     ws_ready: WebSockStatus::Connected,
    //     accepted: Accepted::Accepted,
    //     front: Some(
    //         m::PlayState(m::playstate::PlayState {
    //             title: "hejsan hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej hej".to_string(),
    //             pause: true,
    //             progress: protocol::util::not_nan_or_zero(0.0),
    //             length: protocol::util::not_nan_or_zero(0.0),
    //             volume: protocol::util::not_nan_or_zero(0.0),
    //             chapter: None,
    //             subtitles: vec![
    //                 m::playstate::Track {id: 0, selected: false, title: "None".to_string()},
    //                 m::playstate::Track {id: 1, selected: false, title: "Engelska".to_string()},
    //                 m::playstate::Track {id: 2, selected: true, title: "Svenska".to_string()},
    //                 m::playstate::Track {id: 3, selected: false, title: "Franska".to_string()},
    //             ],
    //             audios: vec![
    //                 m::playstate::Track {id: 0, selected: false, title: "None".to_string()},
    //                 m::playstate::Track {id: 1, selected: false, title: "Engelska".to_string()},
    //                 m::playstate::Track {id: 2, selected: true, title: "Japanska".to_string()},
    //             ],
    //         })
    //         .into(),
    //     ), // front: Some(
    // //     fs::Refreshing(fs::Refreshing {
    // //         roots: vec![
    // //             fs::RootInfo {
    // //                 path: "root1".to_string(),
    // //                 status: fs::RootStatus::Loading,
    // //             },
    // //             fs::RootInfo {
    // //                 path: "root2".to_string(),
    // //                 status: fs::RootStatus::Pending,
    // //             },
    // //         ],
    // //         total_dirs: 80,
    // //         done_dirs: 20,
    // //         num_errors: 5,
    // //     })
    // //     fs::Results(fs::Results{
    // //         query: "testing testing".to_string(),
    // //         query_valid: true,
    // //         results: vec![fs::SearchResult{path: "/anime_cache/[EMBER] Go-Toubun no Hanayome (Movie) [1080p] [HEVC WEBRip DDP].mkv".to_string(), root: 0, indices: vec![1, 10, 21], basename: 13}],
    // //     })
    // // .into(),
    // // )
    // }).render();
}
