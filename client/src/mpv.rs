use protocol::{
    to_client::front::mpv as prot,
    to_server::{mpvcontrol, mpvstart},
};
use yew::prelude::*;

use crate::progressbar::Progressbar;
use crate::WebSockStatus;

#[derive(Properties, PartialEq)]
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
    let play_icon = play_icon(&props.front);

    html! {
        <article class={classes!("stacker")}>
            if matches!(props.front, prot::Mpv::Load) {
                <div>{"Loading"}</div>
            }
            <button onclick={click_send!(mpvstart::Stop)}
                    disabled={active.is_disconnected()}
                    class={classes!("icon", "icon-close", "icon-right", "left", "error")}>
                {"Exit"}
            </button>
            <div class={classes!("left", "pad")}>
                <span>{progress_min}{"/"}{length_min}</span>
                <span class={classes!("float-right")}>{chapter}{"/"}{chapter_total}</span>
            </div>
            <div class={classes!("pad")}>
                <Progressbar progress={progress(&props.front)}
                             outer_class={classes!("mpv-progress-outer")}
                             inner_class={classes!("mpv-progress-inner")}/>
            </div>
            <div class={classes!("space-evenly")}>
                <button onclick={click_send!(mpvcontrol::PrevChapter)}
                        class={classes!("round", "icon", "icon-skip-back")}
                        disabled={!clickable} />

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
                        disabled={!clickable} />
            </div>
            <div class={classes!("space-evenly")}>
                <button onclick={click_send!(mpvcontrol::CycleSub)}
                        class={classes!("round", "icon", "icon-subtitles")}
                        disabled={!clickable} />
                <button onclick={click_send!(mpvcontrol::CycleAudio)}
                        class={classes!("round", "icon", "icon-audio-file")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::VolumeUp)}
                        class={classes!("round", "icon", "icon-volume-up")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::VolumeDown)}
                        class={classes!("round", "icon", "icon-volume-down")}
                        disabled={!clickable} />

                <button onclick={click_send!(mpvcontrol::ToggleMute)}
                        class={classes!("round", "icon", "icon-volume-mute")}
                        disabled={!clickable} />
            </div>
            <div class={classes!("section", "pad", "small")}>
                <span>{"Subtitle controls"}</span>
            </div>
            <div class={classes!("space-evenly")}>
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
        </article>
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

fn progress(front: &prot::Mpv) -> u8 {
    let p = match *front {
        prot::PlayState(prot::PlayState {
            progress, length, ..
        }) if length != 0.0 => ((progress / length) * 100.0).into_inner(),
        _ => 0.0,
    }
    .trunc() as u8;

    if p > 100 {
        100
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

fn play_icon(front: &prot::Mpv) -> Option<&'static str> {
    match front {
        prot::Load => None,
        prot::PlayState(prot::PlayState { pause: true, .. }) => Some("icon-play"),
        prot::PlayState(prot::PlayState { pause: false, .. }) => Some("icon-pause"),
    }
}
