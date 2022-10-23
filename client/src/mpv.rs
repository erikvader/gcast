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

    let (progress_min, length_min) = match props.front {
        prot::Load => ("0".to_string(), "0".to_string()),
        prot::PlayState(prot::PlayState {
            progress, length, ..
        }) => (
            timestamp(progress.into_inner()),
            timestamp(length.into_inner()),
        ),
    };

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
                {progress_min}{"/"}{length_min}
            </div>
            <div class={classes!("pad")}>
                <Progressbar progress={progress(props)}
                             outer_class={classes!("mpv-progress-outer")}
                             inner_class={classes!("mpv-progress-inner")}/>
            </div>
            <button onclick={click_send!(mpvcontrol::TogglePause)}
                    disabled={!clickable}>
                {"Pause/play"}
            </button>
            <button onclick={click_send!(mpvcontrol::CycleAudio)}
                    disabled={!clickable}>
                {"CycleAudio"}
            </button>
            <button onclick={click_send!(mpvcontrol::VolumeUp)}
                    disabled={!clickable}>
                {"VolumeUp"}
            </button>
            <button onclick={click_send!(mpvcontrol::VolumeDown)}
                    disabled={!clickable}>
                {"VolumeDown"}
            </button>
            <button onclick={click_send!(mpvcontrol::ToggleMute)}
                    disabled={!clickable}>
                {"ToggleMute"}
            </button>
            <button onclick={click_send!(mpvcontrol::SubDelayEarlier)}
                    disabled={!clickable}>
                {"SubDelayEarlier"}
            </button>
            <button onclick={click_send!(mpvcontrol::SubDelayLater)}
                    disabled={!clickable}>
                {"SubDelayLater"}
            </button>
            <button onclick={click_send!(mpvcontrol::NextChapter)}
                    disabled={!clickable}>
                {"NextChapter"}
            </button>
            <button onclick={click_send!(mpvcontrol::PrevChapter)}
                    disabled={!clickable}>
                {"PrevChapter"}
            </button>
            <button onclick={click_send!(mpvcontrol::SeekBack)}
                    disabled={!clickable}>
                {"SeekBack"}
            </button>
            <button onclick={click_send!(mpvcontrol::SeekForward)}
                    disabled={!clickable}>
                {"SeekForward"}
            </button>
            <button onclick={click_send!(mpvcontrol::SeekBackLong)}
                    disabled={!clickable}>
                {"SeekBackLong"}
            </button>
            <button onclick={click_send!(mpvcontrol::SeekForwardLong)}
                    disabled={!clickable}>
                {"SeekForwardLong"}
            </button>
            <button onclick={click_send!(mpvcontrol::CycleSub)}
                    disabled={!clickable}>
                {"CycleSub"}
            </button>
            <button onclick={click_send!(mpvcontrol::SubLarger)}
                    disabled={!clickable}>
                {"SubLarger"}
            </button>
            <button onclick={click_send!(mpvcontrol::SubSmaller)}
                    disabled={!clickable}>
                {"SubSmaller"}
            </button>
            <button onclick={click_send!(mpvcontrol::SubMoveUp)}
                    disabled={!clickable}>
                {"SubMoveUp"}
            </button>
            <button onclick={click_send!(mpvcontrol::SubMoveDown)}
                    disabled={!clickable}>
                {"SubMoveDown"}
            </button>
        </article>
    }
}

fn progress(props: &MpvProps) -> u8 {
    let p = match props.front {
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

fn timestamp(seconds: f64) -> String {
    if seconds.is_nan() || seconds.is_infinite() || seconds < 0.0 {
        log::warn!("Got weird value as seconds: {}", seconds);
        "??:??:??".to_string()
    } else {
        let hours = seconds / 3600.0;
        let minutes = (seconds % 3600.0) / 60.0;
        let s = seconds % 60.0;
        format!("{:02.0}:{:02.0}:{:02.0}", hours, minutes, s)
    }
}
