use std::time::Duration;

use protocol::{
    to_client::front::mpv as prot,
    to_server::{mpvcontrol, mpvstart},
    util::{Percent, PositivePercent},
};
use yew::prelude::*;

use crate::{
    back_button::{BackButton, Type},
    hooks::server::UseServer,
};
use crate::{debounce::use_debounce, progressbar::ProgressbarInteractive};

#[derive(Properties, PartialEq, Eq)]
pub struct MpvProps {
    pub front: prot::Mpv,
}

#[rustfmt::skip::macros(html)]
#[function_component(Mpv)]
pub fn mpv(props: &MpvProps) -> Html {
    let server = use_context::<UseServer>().expect("no server context found");
    let clickable = server.is_connected() && !matches!(props.front, prot::Mpv::Load);

    let (progress_min, length_min) = progress_timestamps(&props.front);
    let (chapter, chapter_total) = chapters(&props.front);
    let has_chapters = has_chapters(&props.front);
    let play_icon = play_icon(&props.front);
    let title = title(&props.front);
    let subtitles = subtitles(&props.front);
    let audios = audios(&props.front);
    let volume = volume(&props.front);

    let on_slide = {
        let sender = server.sender();
        use_debounce(Duration::from_millis(250), move |seek| {
            sender.send(mpvcontrol::SeekAbs(seek))
        })
    };

    // TODO: show a volume indicator
    html! {
        <article class={classes!("stacker")}>
            <BackButton button_type={Type::Exit}
                        onclick={click_send!(server, mpvstart::Stop)} />
            // TODO: add a spinning thingy when loading
            <div class={classes!("pad")}>
                <div class={classes!("kinda-big", "mpv-title")}>{title}</div>
            </div>
            <div class={classes!("left", "pad")}>
                <span>{progress_min}{"/"}{length_min}</span>
                <span class={classes!("float-right")}>{chapter}{"/"}{chapter_total}</span>
            </div>
            <div class={classes!("pad")}>
                <ProgressbarInteractive
                             disabled={!clickable}
                             on_slide={on_slide}
                             progress={progress(&props.front)}
                             outer_class={classes!("mpv-progress-outer")}
                             inner_class={classes!("mpv-progress-inner")}/>
            </div>
            <div class={classes!("space-evenly", "pad")}>
                <button onclick={click_send!(server, mpvcontrol::PrevChapter)}
                        class={classes!("round", "icon", "icon-skip-back")}
                        disabled={!clickable || !has_chapters} />

                <button onclick={click_send!(server, mpvcontrol::SeekBackLong)}
                        class={classes!("round", "icon", "icon-backward30")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SeekBack)}
                        class={classes!("round", "icon", "icon-backward5")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::TogglePause)}
                        class={classes!("round", "kinda-big", "icon", play_icon)}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SeekForward)}
                        class={classes!("round", "icon", "icon-forward5")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SeekForwardLong)}
                        class={classes!("round", "icon", "icon-forward30")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::NextChapter)}
                        class={classes!("round", "icon", "icon-skip-fwd")}
                        disabled={!clickable || !has_chapters} />
            </div>
            <div class={classes!("section", "pad", "small")}>
                <span>{"Subtitle controls"}</span>
            </div>
            <div class={classes!("space-evenly", "pad")}>
                <button onclick={click_send!(server, mpvcontrol::SubDelayEarlier)}
                        class={classes!("round", "icon", "icon-back-arrow")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SubDelayLater)}
                        class={classes!("round", "icon", "icon-forward-arrow")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SubLarger)}
                        class={classes!("round", "icon", "icon-add")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SubSmaller)}
                        class={classes!("round", "icon", "icon-remove")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SubMoveUp)}
                        class={classes!("round", "icon", "icon-up-arrow")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::SubMoveDown)}
                        class={classes!("round", "icon", "icon-down-arrow")}
                        disabled={!clickable} />
            </div>
            <TrackSelector tracks={subtitles}
                           disabled={!clickable}
                           onclick={click_send!(server, id -> mpvcontrol::SetSub(id))} />
            <div class={classes!("section", "pad", "small")}>
                <span>{"Audio controls"}</span>
                <span class={classes!("float-right")}>{format!("Volume: {volume}")}</span>
            </div>
            <div class={classes!("space-evenly", "pad")}>
                <button onclick={click_send!(server, mpvcontrol::ToggleMute)}
                        class={classes!("round", "icon", "icon-volume-mute")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::VolumeDown)}
                        class={classes!("round", "icon", "icon-volume-down")}
                        disabled={!clickable} />

                <button onclick={click_send!(server, mpvcontrol::VolumeUp)}
                        class={classes!("round", "icon", "icon-volume-up")}
                        disabled={!clickable} />
            </div>
            <TrackSelector tracks={audios}
                           disabled={!clickable}
                           onclick={click_send!(server, id -> mpvcontrol::SetAudio(id))} />
        </article>
    }
}

#[derive(Properties, PartialEq)]
pub struct TrackSelectorProps {
    pub tracks: Vec<prot::playstate::Track>,
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
            let selected = (!t.selected).then_some("inverted");
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
        <div class={classes!("pad", "fill-nicely", "gap")}>
            {contents}
        </div>
    }
}

fn chapters(front: &prot::Mpv) -> (i64, i64) {
    match *front {
        prot::PlayState(prot::playstate::PlayState {
            chapter: Some((c, t)),
            ..
        }) => (c, t),
        _ => (0, 0),
    }
}

fn has_chapters(front: &prot::Mpv) -> bool {
    matches!(
        *front,
        prot::PlayState(prot::playstate::PlayState {
            chapter: Some(_),
            ..
        })
    )
}

fn progress(front: &prot::Mpv) -> Percent {
    match *front {
        prot::PlayState(prot::playstate::PlayState {
            progress, length, ..
        }) => Percent::of(progress.as_secs_f64(), length.as_secs_f64())
            .unwrap_or(Percent::ZERO),
        _ => Percent::ZERO,
    }
}

fn volume(front: &prot::Mpv) -> String {
    match front {
        prot::PlayState(prot::playstate::PlayState {
            volume: Some(volume),
            ..
        }) => volume.to_string(),
        _ => "muted".to_string(),
    }
}

fn progress_timestamps(front: &prot::Mpv) -> (String, String) {
    match front {
        prot::Load => ("0".to_string(), "0".to_string()),
        prot::PlayState(prot::playstate::PlayState {
            progress, length, ..
        }) => (
            timestamp(progress.as_secs_f64()),
            timestamp(length.as_secs_f64()),
        ),
    }
}

fn timestamp(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
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
        prot::PlayState(prot::playstate::PlayState { title, .. }) => title,
    }
}

fn play_icon(front: &prot::Mpv) -> Vec<&'static str> {
    match front {
        prot::Load => vec!["icon-renew", "spin"],
        prot::PlayState(prot::playstate::PlayState { pause: true, .. }) => {
            vec!["icon-play"]
        }
        prot::PlayState(prot::playstate::PlayState { pause: false, .. }) => {
            vec!["icon-pause"]
        }
    }
}

fn subtitles(front: &prot::Mpv) -> Vec<prot::playstate::Track> {
    match front {
        prot::Load => vec![prot::playstate::Track {
            id: 0,
            selected: true,
            title: "Loading...".to_string(),
        }],
        prot::PlayState(prot::playstate::PlayState { subtitles, .. }) => {
            subtitles.clone()
        }
    }
}

fn audios(front: &prot::Mpv) -> Vec<prot::playstate::Track> {
    match front {
        prot::Load => vec![prot::playstate::Track {
            id: 0,
            selected: true,
            title: "Loading...".to_string(),
        }],
        prot::PlayState(prot::playstate::PlayState { audios, .. }) => audios.clone(),
    }
}
