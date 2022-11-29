pub mod errors;

use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

use derive_builder::Builder;
use libmpv::{
    events::{Event, PropertyData},
    FileState, Format, Mpv,
};
use protocol::{
    to_client::front::mpv::{Mpv as ClientMpv, PlayState},
    to_server::mpvcontrol::MpvControl,
    util::not_nan_or_zero,
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

use crate::util::join_handle_wait_take;

pub use self::errors::*;
pub type MpvResult<T> = Result<T, MpvError>;

type Command = &'static str;
type StateRcv = mpsc::Receiver<MpvResult<MpvState>>;
type StateSnd = mpsc::Sender<MpvResult<MpvState>>;
type HandleResp = oneshot::Sender<MpvResult<()>>;
type HandleSnd = mpsc::Sender<(Command, HandleResp)>;

const EV_CTX_WAIT: f64 = 5.0;
const BANNED_PROPERTIES: &[&str] = &["idle"];

static MPV_THREAD_ON: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct MpvHandle {
    tx: HandleSnd,
    rx: StateRcv,
    joinhandle: JoinHandle<()>,
}

#[derive(Debug)]
pub enum EndReason {
    Eof,
    Stop,
    Quit,
    Error,
    Redirect,
    Unknown(u32),
}

#[derive(Debug, Clone, Builder)]
#[builder(derive(Debug))]
pub struct State {
    title: String,
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
            0 => Eof,
            2 => Stop,
            3 => Quit,
            4 => Error,
            5 => Redirect,
            x => Unknown(x),
        }
    }
}

impl MpvState {
    pub fn to_client_state(&self) -> Option<ClientMpv> {
        use MpvState::*;
        match self {
            Load => Some(ClientMpv::Load),
            End(_) => None,
            Play(state) => Some(ClientMpv::PlayState(PlayState {
                title: state.title.clone(),
                pause: state.pause,
                progress: not_nan_or_zero(state.playback_time),
                length: not_nan_or_zero(state.duration),
                volume: not_nan_or_zero(state.volume),
                chapter: if state.chapters > 0 {
                    Some((state.chapter + 1, state.chapters))
                } else {
                    None
                },
            })),
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
        NextChapter => "add chapter 1", // TODO: crashes when there are no chapters
        PrevChapter => "add chapter -1",
        SeekBack => "seek -5",
        SeekForward => "seek 5",
        CycleSub => "cycle sub",
        SeekBackLong => "seek -30",
        SeekForwardLong => "seek 30",
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

    pub async fn next(&mut self) -> MpvResult<MpvState> {
        self.rx.recv().await.unwrap_or(Err(MpvError::Exited))
    }

    pub async fn wait_until_closed(self) {
        drop(self.tx);
        drop(self.rx);
        log::debug!("Waiting for mpv threads to close");
        join_handle_wait_take(self.joinhandle).await;
    }
}

fn observe_some_properties(ctx: &libmpv::events::EventContext<'_>) -> libmpv::Result<()> {
    ctx.disable_deprecated_events()?;
    ctx.observe_property("media-title", Format::String, 0)?;
    ctx.observe_property("pause", Format::Flag, 0)?;
    ctx.observe_property("playback-time", Format::Double, 0)?; //time-pos, percent-pos, stream-pos, stream-end
    ctx.observe_property("duration", Format::Double, 0)?;
    ctx.observe_property("volume", Format::Double, 0)?;
    ctx.observe_property("chapters", Format::Int64, 0)?;
    ctx.observe_property("chapter", Format::Int64, 0)?;
    Ok(())
}

pub fn mpv(path: &str) -> MpvResult<MpvHandle> {
    if MPV_THREAD_ON.swap(true, Ordering::SeqCst) {
        return Err(MpvError::AlreadyRunning);
    }

    let (h_tx, h_rx): (HandleSnd, _) = mpsc::channel(crate::CHANNEL_SIZE);
    let (s_tx, s_rx): (_, StateRcv) = mpsc::channel(crate::CHANNEL_SIZE);

    let mpv = Mpv::with_initializer(|x| {
        x.set_property("idle", "once")?; // NOTE: needed for the correct events to appear
        for (key, value) in crate::config::mpv_options() {
            if BANNED_PROPERTIES.contains(&key.as_str()) {
                log::warn!("It is not allowed to change '{}'", key);
                continue;
            }

            if let Err(e) = x.set_property(&key, &*value) {
                log::error!(
                    "Failed to set mpv property '{}' to '{}' because '{}'",
                    key,
                    value,
                    e
                );
            }
        }
        Ok(())
    })?;

    log::debug!("loading file: {}", path);
    mpv.playlist_load_files(&[(path, FileState::Replace, None)])?;

    let joinhandle = tokio::task::spawn_blocking(move || {
        let barrier = std::sync::Barrier::new(2);
        thread::scope(|s| {
            s.spawn(|| {
                let mut h_rx = h_rx;
                barrier.wait();
                while let Some((cmd, tx_res)) = h_rx.blocking_recv() {
                    log::debug!("Executing mpv command: {}", cmd);
                    let res = mpv.command(cmd, &[]); // NOTE: args is just appended to cmd
                    if let Err(e) = &res {
                        log::error!("Mpv errored: {}", e); // TODO: remove since handle probably also will log?
                    }
                    tx_res.send(res.map_err(|e| e.into())).ok();
                }
                log::debug!("Mpv handle thread shut down");
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

                log::debug!("Mpv state thread shut down");
            });
        });

        MPV_THREAD_ON.store(false, Ordering::SeqCst);
        log::debug!("Mpv thread shut down");
    });

    Ok(MpvHandle {
        tx: h_tx,
        rx: s_rx,
        joinhandle,
    })
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

fn take_string(data: &PropertyData) -> String {
    if let PropertyData::Str(s) = data {
        return s.to_string();
    }
    panic!("'{:?}' is not a string", data)
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
                log::debug!("Change: {} to {:?}", name, change);
                match name {
                    "media-title" => partial.title(take_string(&change)),
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
                    "media-title" => state.title = take_string(&change),
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
