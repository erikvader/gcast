use protocol::{
    to_client::front::mpv as prot,
    to_server::{mpvcontrol, mpvstart},
};
use yew::prelude::*;

use crate::progressbar::Progressbar;
use crate::WebSockStatus;
use crate::{
    back_button::{BackButton, Type},
    websocket::websocket_send,
};

#[derive(Properties, PartialEq, Eq)]
pub struct MpvProps {
    pub front: prot::Mpv,
}

#[rustfmt::skip::macros(html)]
#[function_component(Mpv)]
pub fn mpv(props: &MpvProps) -> Html {
    let active = use_context::<WebSockStatus>().expect("no active context found");
    let clickable = active.is_connected() && !matches!(props.front, prot::Mpv::Load);

    let (progress_min, length_min) = progress_timestamps(&props.front);
    let (chapter, chapter_total) = chapters(&props.front);
    let has_chapters = has_chapters(&props.front);
    let play_icon = play_icon(&props.front);
    let title = title(&props.front);
    let subtitles = subtitles(&props.front);
    let audios = audios(&props.front);

    // TODO: show a volume indicator
    html! {
        <article class={classes!("stacker")}>
            <BackButton button_type={Type::Exit}
                        onclick={click_send!(mpvstart::Stop)} />
            // TODO: add a spinning thingy when loading
            <div class={classes!("pad")}>
                <div class={classes!("kinda-big", "mpv-title")}>{title}</div>
            </div>
            <div class={classes!("left", "pad")}>
                <span>{progress_min}{"/"}{length_min}</span>
                <span class={classes!("float-right")}>{chapter}{"/"}{chapter_total}</span>
            </div>
            <div class={classes!("pad")}>
                <Progressbar progress={progress(&props.front)}
                             outer_class={classes!("mpv-progress-outer")}
                             inner_class={classes!("mpv-progress-inner")}/>
            </div>
            <div class={classes!("space-evenly", "pad")}>
                <button onclick={click_send!(mpvcontrol::PrevChapter)}
                        class={classes!("round", "icon", "icon-skip-back")}
                        disabled={!clickable || !has_chapters} />

                <button onclick={click_send!(mpvcontrol::SeekBackLong)}
                        class={classes!("round", "icon", "icon-backward30")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SeekBack)}
                        class={classes!("round", "icon", "icon-backward5")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::TogglePause)}
                        class={classes!("round", "kinda-big", "icon", play_icon)}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SeekForward)}
                        class={classes!("round", "icon", "icon-forward5")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SeekForwardLong)}
                        class={classes!("round", "icon", "icon-forward30")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::NextChapter)}
                        class={classes!("round", "icon", "icon-skip-fwd")}
                        disabled={!clickable || !has_chapters} />
            </div>
            <div class={classes!("section", "pad", "small")}>
                <span>{"subtitle controls"}</span>
            </div>
            <div class={classes!("space-evenly", "pad")}>
                <button onclick={click_send!(mpvcontrol::SubDelayEarlier)}
                        class={classes!("round", "icon", "icon-back-arrow")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SubDelayLater)}
                        class={classes!("round", "icon", "icon-forward-arrow")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SubLarger)}
                        class={classes!("round", "icon", "icon-add")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SubSmaller)}
                        class={classes!("round", "icon", "icon-remove")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SubMoveUp)}
                        class={classes!("round", "icon", "icon-up-arrow")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::SubMoveDown)}
                        class={classes!("round", "icon", "icon-down-arrow")}
                        disabled={!clickable} />
            </div>
            <TrackSelector tracks={subtitles}
                           disabled={!clickable}
                           onclick={Callback::from(|id| websocket_send(mpvcontrol::SetSub(id)))} />
            <div class={classes!("section", "pad", "small")}>
                <span>{"Audio controls"}</span>
            </div>
            <div class={classes!("space-evenly", "pad")}>
                <button onclick={click_send!(mpvcontrol::ToggleMute)}
                        class={classes!("round", "icon", "icon-volume-mute")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::VolumeDown)}
                        class={classes!("round", "icon", "icon-volume-down")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::VolumeUp)}
                        class={classes!("round", "icon", "icon-volume-up")}
                        disabled={!clickable} />
            </div>
            <TrackSelector tracks={audios}
                           disabled={!clickable}
                           onclick={Callback::from(|id| websocket_send(mpvcontrol::SetAudio(id)))} />
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub struct TrackSelectorProps {
    pub tracks: Vec<prot::Track>,
    pub onclick: Callback<i64>,
    pub disabled: bool,
}

#[rustfmt::skip::macros(html)]
#[function_component(TrackSelector)]
pub fn track_selector(props: &TrackSelectorProps) -> Html {
    let contents: Html = props
        .tracks
        .iter()
        .map(|t| {
            let onclick = props.onclick.clone();
            let id = t.id;
            let selected = if t.selected { None } else { Some("inverted") };
            html! {
                <button onclick={Callback::from(move |_| onclick.emit(id))}
                        disabled={props.disabled}
                        class={classes!(selected)}>
                    {t.title.clone()}
                </button>
            }
        })
        .collect();

    html! {
        <div class={classes!("pad", "box", "fill-nicely")}>
            {contents}
        </div>
    }
}

fn chapters(front: &prot::Mpv) -> (i64, i64) {
    match *front {
        prot::PlayState(prot::PlayState {
            chapter: Some((c, t)),
            ..
        }) => (c, t),
        _ => (0, 0),
    }
}

fn has_chapters(front: &prot::Mpv) -> bool {
    matches!(
        *front,
        prot::PlayState(prot::PlayState {
            chapter: Some(_),
            ..
        })
    )
}

fn progress(front: &prot::Mpv) -> f64 {
    let p = match *front {
        prot::PlayState(prot::PlayState {
            progress, length, ..
        }) if length != 0.0 => ((progress / length) * 100.0).into_inner(),
        _ => 0.0,
    };

    if p > 100.0 {
        100.0
    } else {
        p
    }
}

fn progress_timestamps(front: &prot::Mpv) -> (String, String) {
    match front {
        prot::Load => ("0".to_string(), "0".to_string()),
        prot::PlayState(prot::PlayState {
            progress, length, ..
        }) => (
            timestamp(progress.into_inner()),
            timestamp(length.into_inner()),
        ),
    }
}

fn timestamp(seconds: f64) -> String {
    if seconds.is_nan() || seconds.is_infinite() || seconds < 0.0 {
        "??:??:??".to_string()
    } else {
        let int = seconds as u64;
        let hours = int / 3600;
        let minutes = (int % 3600) / 60;
        let s = int % 60;
        format!("{:02.0}:{:02.0}:{:02.0}", hours, minutes, s)
    }
}

fn title(front: &prot::Mpv) -> &str {
    match front {
        prot::Load => "Loading...",
        prot::PlayState(prot::PlayState { title, .. }) => title,
    }
}

fn play_icon(front: &prot::Mpv) -> Option<&'static str> {
    match front {
        prot::Load => None,
        prot::PlayState(prot::PlayState { pause: true, .. }) => Some("icon-play"),
        prot::PlayState(prot::PlayState { pause: false, .. }) => Some("icon-pause"),
    }
}

fn subtitles(front: &prot::Mpv) -> Vec<prot::Track> {
    match front {
        prot::Load => vec![prot::Track {
            id: 0,
            selected: true,
            title: "Loading...".to_string(),
        }],
        prot::PlayState(prot::PlayState { subtitles, .. }) => subtitles.clone(),
    }
}

fn audios(front: &prot::Mpv) -> Vec<prot::Track> {
    match front {
        prot::Load => vec![prot::Track {
            id: 0,
            selected: true,
            title: "Loading...".to_string(),
        }],
        prot::PlayState(prot::PlayState { audios, .. }) => audios.clone(),
    }
}
