pub mod errors;

use std::thread;

use derive_builder::Builder;
use libmpv::{
    events::{Event, PropertyData},
    FileState, Format, Mpv,
};
use protocol::{
    to_client::front::mpv::{Mpv as ClientMpv, PlayState},
    to_server::mpvcontrol::MpvControl,
};
use tokio::sync::{mpsc, oneshot};

pub use self::errors::*;

type Command = &'static str;
type MpvResult<T> = Result<T, MpvError>;
type StateRcv = mpsc::Receiver<MpvResult<MpvState>>;
type StateSnd = mpsc::Sender<MpvResult<MpvState>>;
type HandleResp = oneshot::Sender<MpvResult<()>>;
type HandleSnd = mpsc::Sender<(Command, HandleResp)>;

const EV_CTX_WAIT: f64 = 5.0;

#[derive(Debug)]
pub struct MpvHandle {
    tx: HandleSnd,
}

#[derive(Debug)]
pub struct MpvStateHandle {
    rx: StateRcv,
}

#[derive(Debug)]
pub enum EndReason {
    EOF,
    Stop,
    Quit,
    Error,
    Redirect,
    Unknown(u32),
}

#[derive(Debug, Clone, Builder)]
#[builder(derive(Debug))]
pub struct State {
    pause: bool,
    playback_time: f64,
    duration: f64,
    volume: f64,
    chapters: i64,
    #[builder(default = "-1")]
    chapter: i64,
}

#[derive(Debug)]
pub enum MpvState {
    Load,
    Play(State),
    End(EndReason),
}

impl From<u32> for EndReason {
    fn from(r: u32) -> Self {
        use EndReason::*;
        match r {
            0 => EOF,
            2 => Stop,
            3 => Quit,
            4 => Error,
            5 => Redirect,
            x => Unknown(x),
        }
    }
}

impl MpvStateHandle {
    pub async fn next(&mut self) -> MpvResult<MpvState> {
        self.rx.recv().await.unwrap_or(Err(MpvError::Exited))
    }
}

impl MpvState {
    pub fn to_client_state(&self) -> Option<ClientMpv> {
        use MpvState::*;
        match self {
            Load => Some(ClientMpv::Load),
            End(_) => None,
            Play(state) => Some(ClientMpv::PlayState(PlayState::new(
                state.pause,
                state.playback_time,
                state.duration,
                state.volume,
                state.chapters,
                state.chapter,
            ))),
        }
    }
}

fn control_string(ctrl: &MpvControl) -> Command {
    use MpvControl::*;
    match ctrl {
        TogglePause => "cycle pause",
        CycleAudio => "cycle audio",
        VolumeUp => "add volume 2",
        VolumeDown => "add volume -2",
        ToggleMute => "cycle mute",
        SubDelayEarlier => "add sub-delay -0.1",
        SubDelayLater => "add sub-delay 0.1",
        NextChapter => "add chapter 1",
        PrevChapter => "add chapter -1",
        SeekBack => "seek -5",
        SeekForward => "seek 5",
        CycleSub => "cycle sub",
        SeekBackLong => "seek -90",
        SeekForwardLong => "seek 90",
        SubLarger => "add sub-scale 0.1",
        SubSmaller => "add sub-scale -0.1",
        SubMoveUp => "add sub-pos -1",
        SubMoveDown => "add sub-pos 1",
    }
}

impl MpvHandle {
    async fn command_str(&self, cmd: Command) -> MpvResult<()> {
        let (tx, rx) = oneshot::channel();
        if self.tx.send((cmd, tx)).await.is_err() {
            return Err(MpvError::Exited);
        }
        match rx.await {
            Ok(res) => res,
            Err(_) => Err(MpvError::Exited),
        }
    }

    pub async fn command(&self, cmd: &MpvControl) -> MpvResult<()> {
        self.command_str(control_string(cmd)).await
    }

    pub async fn quit(&self) -> MpvResult<()> {
        self.command_str("quit").await
    }
}

fn observe_some_properties(ctx: &libmpv::events::EventContext<'_>) -> libmpv::Result<()> {
    ctx.disable_deprecated_events()?;
    ctx.observe_property("pause", Format::Flag, 0)?;
    ctx.observe_property("playback-time", Format::Double, 0)?; //time-pos, percent-pos, stream-pos, stream-end
    ctx.observe_property("duration", Format::Double, 0)?;
    ctx.observe_property("volume", Format::Double, 0)?;
    ctx.observe_property("chapters", Format::Int64, 0)?;
    ctx.observe_property("chapter", Format::Int64, 0)?;
    Ok(())
}

pub fn mpv(path: &str) -> MpvResult<(MpvHandle, MpvStateHandle)> {
    let (h_tx, h_rx): (HandleSnd, _) = mpsc::channel(1000);
    let (s_tx, s_rx): (_, StateRcv) = mpsc::channel(1000);

    let mpv = Mpv::with_initializer(|x| {
        x.set_property("force-window", "immediate")?;
        x.set_property("idle", "once")?;
        // x.set_property("fullscreen", true)?;
        Ok(())
    })?;

    log::debug!("loading file: {}", path);
    mpv.playlist_load_files(&[(path, FileState::Replace, None)])?;

    thread::spawn(move || {
        let barrier = std::sync::Barrier::new(2);
        thread::scope(|s| {
            s.spawn(|| {
                let mut h_rx = h_rx;
                barrier.wait();
                while let Some((cmd, tx_res)) = h_rx.blocking_recv() {
                    log::debug!("executing mpv command: {}", cmd);
                    let res = mpv.command(cmd, &[]); // NOTE: args is just appended to cmd
                    if let Err(e) = &res {
                        log::error!("mpv errored: {}", e); // TODO: remove since handle probably also will log?
                    }
                    tx_res.send(res.map_err(|e| e.into())).ok();
                }
                log::debug!("Mpv handle thread shutting down");
            });

            s.spawn(|| {
                let s_tx = s_tx;
                let mut ev_ctx = mpv.create_event_context();
                let res = observe_some_properties(&ev_ctx);
                barrier.wait();
                if let Err(e) = res {
                    log::error!(
                        "Failed to observe properties, shutting down state thread: {}",
                        e
                    );
                    s_tx.blocking_send(Err(e.into())).ok();
                    return;
                }

                if let Some(state) = wait_for_play(&s_tx, &mut ev_ctx) {
                    wait_for_end(&s_tx, &mut ev_ctx, state);
                }

                log::debug!("Mpv state thread shutting down");
            });
        });
        log::debug!("Mpv thread shutting down");
    });

    Ok((MpvHandle { tx: h_tx }, MpvStateHandle { rx: s_rx }))
}

fn take_flag(data: &PropertyData) -> bool {
    if let PropertyData::Flag(b) = data {
        return *b;
    }
    panic!("'{:?}' is not a flag", data)
}

fn take_double(data: &PropertyData) -> f64 {
    if let PropertyData::Double(d) = data {
        return *d;
    }
    panic!("'{:?}' is not a double", data)
}

fn take_int(data: &PropertyData) -> i64 {
    if let PropertyData::Int64(i) = data {
        return *i;
    }
    panic!("'{:?}' is not an int", data)
}

fn wait_for_play(
    s_tx: &StateSnd,
    ev_ctx: &mut libmpv::events::EventContext,
) -> Option<State> {
    s_tx.blocking_send(Ok(MpvState::Load)).ok();
    let mut partial = StateBuilder::default();

    loop {
        if s_tx.is_closed() {
            return None;
        }

        match ev_ctx.wait_event(EV_CTX_WAIT) {
            None => (),
            Some(Ok(Event::Shutdown)) => {
                s_tx.blocking_send(Ok(MpvState::End(EndReason::Quit))).ok();
                return None;
            }
            Some(Ok(Event::EndFile(r))) => {
                s_tx.blocking_send(Ok(MpvState::End(r.into()))).ok();
                return None;
            }
            Some(Err(e)) => {
                s_tx.blocking_send(Err(e.into())).ok();
                return None;
            }
            Some(Ok(Event::PropertyChange { name, change, .. })) => {
                log::debug!("change: {} to {:?}", name, change);
                match name {
                    "pause" => partial.pause(take_flag(&change)),
                    "playback-time" => partial.playback_time(take_double(&change)),
                    "duration" => partial.duration(take_double(&change)),
                    "volume" => partial.volume(take_double(&change)),
                    "chapters" => partial.chapters(take_int(&change)),
                    "chapter" => partial.chapter(take_int(&change)),
                    _ => &mut partial,
                };

                if let Ok(state) = partial.build() {
                    return Some(state);
                }
            }
            Some(Ok(_)) => (),
        }
    }
}

fn wait_for_end(
    s_tx: &mpsc::Sender<Result<MpvState, MpvError>>,
    ev_ctx: &mut libmpv::events::EventContext,
    mut state: State,
) {
    s_tx.blocking_send(Ok(MpvState::Play(state.clone()))).ok();
    loop {
        if s_tx.is_closed() {
            return;
        }

        match ev_ctx.wait_event(EV_CTX_WAIT) {
            None => (),
            Some(Ok(Event::Shutdown)) => {
                s_tx.blocking_send(Ok(MpvState::End(EndReason::Quit))).ok();
                return;
            }
            Some(Ok(Event::EndFile(r))) => {
                s_tx.blocking_send(Ok(MpvState::End(r.into()))).ok();
                return;
            }
            Some(Err(e)) => {
                s_tx.blocking_send(Err(e.into())).ok();
                return;
            }
            Some(Ok(Event::PropertyChange { name, change, .. })) => {
                match name {
                    "pause" => state.pause = take_flag(&change),
                    "playback-time" => state.playback_time = take_double(&change),
                    "duration" => state.duration = take_double(&change),
                    "volume" => state.volume = take_double(&change),
                    "chapters" => state.chapters = take_int(&change),
                    "chapter" => state.chapter = take_int(&change),
                    _ => (),
                };

                s_tx.blocking_send(Ok(MpvState::Play(state.clone()))).ok();
            }
            Some(Ok(_)) => (),
        }
    }
}
