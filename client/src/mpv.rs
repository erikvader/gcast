use protocol::{
    to_client::front::mpv as prot,
    to_server::{mpvcontrol, mpvstart},
};
use yew::prelude::*;

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

    html! {
        <>
            if matches!(props.front, prot::Mpv::Load) {
                <div>{"Loading"}</div>
            }
            <button onclick={click_send!(mpvstart::Stop)}
                    disabled={active.is_disconnected()}>
                {"Exit"}
            </button>
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
        </>
    }
}
