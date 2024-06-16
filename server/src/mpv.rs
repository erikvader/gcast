use anyhow::Context;
use libmpv::{EndReason, Event, LogLevel, PropertyValue};
use std::{mem, time::Duration};

use protocol::{
    to_client::front::mpv::{
        playstate::{PlayState, Track as ClientTrack},
        Mpv as ClientMpv,
    },
    to_server::mpvcontrol::MpvControl,
    util::Percent,
};
use tokio::task::spawn_blocking;

use crate::{config, mpv::lang::HumanLang, util::join_handle_wait_take};

use self::lang::{AutoLang, Lang};

mod lang;

pub type MpvResult<T> = anyhow::Result<T>;

const DEF_USR: u64 = 0;

pub struct MpvHandle {
    handle: libmpv::Handle<libmpv::Async>,
    state: MpvState,
    auto_lang: AutoLang,
}

#[derive(Debug, Clone)]
struct State {
    title: String,
    pause: bool,
    playback_time: f64,
    duration: f64,
    volume: f64,
    chapters: i64,
    chapter: i64,
    tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Track {
    id: i64,
    ttype: TrackType,
    lang: Lang,
    selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrackType {
    Audio,
    Video,
    Sub,
}

#[derive(Debug)]
enum MpvState {
    Load,
    Play(State),
    End(EndReason),
}

impl AsRef<Lang> for Track {
    fn as_ref(&self) -> &Lang {
        &self.lang
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            title: "".to_string(),
            pause: true,
            playback_time: 0.0,
            duration: 0.0,
            volume: 0.0,
            chapters: 0,
            chapter: 0,
            tracks: Vec::new(),
        }
    }
}

impl MpvState {
    fn to_client_state(&self) -> Option<ClientMpv> {
        match self {
            MpvState::Load => Some(ClientMpv::Load),
            MpvState::End(_) => None,
            MpvState::Play(state) => Some(ClientMpv::PlayState(PlayState {
                title: state.title.clone(),
                pause: state.pause,
                progress: Duration::try_from_secs_f64(state.playback_time)
                    .unwrap_or(Duration::ZERO),
                length: Duration::try_from_secs_f64(state.duration)
                    .unwrap_or(Duration::ZERO),
                volume: Percent::try_new(state.volume).unwrap_or(Percent::ZERO),
                chapter: if state.chapters > 0 {
                    Some((state.chapter + 1, state.chapters))
                } else {
                    None
                },
                subtitles: to_client_tracks(&state.tracks, TrackType::Sub),
                audios: to_client_tracks(&state.tracks, TrackType::Audio),
            })),
        }
    }
}

fn to_client_tracks(tracks: &[Track], ttype: TrackType) -> Vec<ClientTrack> {
    let mut client_tracks: Vec<_> = tracks
        .iter()
        .filter(|t| t.ttype == ttype)
        .map(|t| ClientTrack {
            id: t.id,
            title: t.lang.to_string(),
            selected: t.selected,
        })
        .collect();

    client_tracks.insert(
        0,
        ClientTrack {
            id: 0,
            title: "None".to_string(),
            selected: client_tracks.iter().all(|t| !t.selected),
        },
    );

    client_tracks
}

impl MpvHandle {
    pub fn command(&mut self, cmd: MpvControl) -> MpvResult<()> {
        let short = Duration::from_secs(5);
        let long = Duration::from_secs(30);
        let vol = 2.0;
        let delay = 0.1;
        let scale = 0.1;
        let pos = 1.0;

        match cmd {
            MpvControl::TogglePause => self.handle.toggle_pause().asynch(DEF_USR)?,
            MpvControl::SetAudio(id) => self.handle.set_audio(id).asynch(DEF_USR)?,
            MpvControl::VolumeUp => self.handle.add_volume(vol).asynch(DEF_USR)?,
            MpvControl::VolumeDown => self.handle.add_volume(-vol).asynch(DEF_USR)?,
            MpvControl::ToggleMute => self.handle.toggle_mute().asynch(DEF_USR)?,
            MpvControl::SubDelayEarlier => {
                self.handle.add_sub_delay(-delay).asynch(DEF_USR)?
            }
            MpvControl::SubDelayLater => {
                self.handle.add_sub_delay(delay).asynch(DEF_USR)?
            }
            MpvControl::NextChapter => self.handle.add_chapter(1).asynch(DEF_USR)?,
            MpvControl::PrevChapter => self.handle.add_chapter(-1).asynch(DEF_USR)?,
            MpvControl::SeekBack => self.handle.seek_backward(short).asynch(DEF_USR)?,
            MpvControl::SeekForward => self.handle.seek_forward(short).asynch(DEF_USR)?,
            MpvControl::SeekBackLong => {
                self.handle.seek_backward(long).asynch(DEF_USR)?
            }
            MpvControl::SeekForwardLong => {
                self.handle.seek_forward(long).asynch(DEF_USR)?
            }
            MpvControl::SetSub(id) => self.handle.set_sub(id).asynch(DEF_USR)?,
            MpvControl::SubLarger => self.handle.add_sub_scale(scale).asynch(DEF_USR)?,
            MpvControl::SubSmaller => {
                self.handle.add_sub_scale(-scale).asynch(DEF_USR)?
            }
            MpvControl::SubMoveUp => self.handle.add_sub_pos(-pos).asynch(DEF_USR)?,
            MpvControl::SubMoveDown => self.handle.add_sub_pos(pos).asynch(DEF_USR)?,
        };

        Ok(())
    }

    pub async fn next(&mut self) -> Option<MpvResult<ClientMpv>> {
        if let MpvState::End(_) = self.state {
            return None;
        }

        loop {
            let event = self.handle.wait_event_async().await;
            log::trace!("Mpv event: {event:?}");
            match event {
                Event::None => (),
                Event::Shutdown => {
                    self.state = MpvState::End(EndReason::Quit);
                    break None;
                }
                Event::Log {
                    prefix,
                    level,
                    text,
                } => {
                    let level = mpv_log_level_convert(level);
                    if let Some(level) = level {
                        log::log!(level, "[{level}] {prefix}: {text}");
                    }
                }
                Event::QueueOverflow => log::error!("Mpv queue overflow"),
                Event::PropertyChange(propvalue) => {
                    if let MpvState::Play(play) = &mut self.state {
                        let updated = play.update_state(&propvalue);

                        if self.auto_lang.has_not_chosen()
                            && matches!(propvalue, PropertyValue::TrackList(_))
                        {
                            if let Err(e) = self
                                .auto_lang
                                .auto_choose(&mut self.handle, &play.tracks)
                                .context("auto choosing tracks")
                            {
                                break Some(Err(e));
                            }
                        }

                        if updated {
                            break Some(Ok(self
                                .state
                                .to_client_state()
                                .expect("not the end")));
                        }
                    }
                }
                Event::PropertyChangeError { format, property } => {
                    log::warn!(
                        "Mpv property change error, format={format:?}, property={property:?}"
                    );
                }
                Event::StartFile => (),
                Event::FileLoaded => {
                    // TODO: how to avoid sending many state updates in rapid succession
                    // after all properties come in one after another?
                    if let Err(e) = observe_properties(&mut self.handle)
                        .context("observing properties")
                    {
                        break Some(Err(e));
                    }
                    self.state = MpvState::Play(State::default());
                    break Some(Ok(self.state.to_client_state().expect("is not end")));
                }
                Event::EndFile { reason, error } => {
                    log::info!("Mpv exited because: {reason:?} (error: {error:?})");
                    self.state = MpvState::End(reason);
                    if let Some(error) = error {
                        break Some(Err(anyhow::anyhow!(
                            "mpv exited with error: {error}"
                        )));
                    }
                    break None;
                }
                Event::GetProperty { .. } => (),
                Event::GetPropertyError {
                    error,
                    format,
                    property,
                    userdata,
                } => {
                    break Some(Err(
                        anyhow::anyhow!("mpv get property error: property={property:?}, format={format:?}, userdata={userdata}, error='{error}'")
                    ));
                }
                Event::SetProperty { error, userdata } => {
                    if let Some(error) = error {
                        break Some(Err(anyhow::anyhow!("mpv set property error: userdata={userdata}, error='{error}'")));
                    }
                }
                Event::Command { error, userdata } => {
                    if let Some(error) = error {
                        break Some(Err(anyhow::anyhow!(
                            "mpv command error: userdata={userdata}, error='{error}'"
                        )));
                    }
                }
                Event::UnsupportedEvent(_) => (),
            }
        }
    }

    pub async fn wait_until_closed(self) -> EndReason {
        let reason = match self.state {
            MpvState::Play(_) | MpvState::Load => EndReason::Quit,
            MpvState::End(reason) => reason,
        };

        join_handle_wait_take(spawn_blocking(move || {
            drop(self);
        }))
        .await;

        reason
    }
}

// TODO: create a MpvOptions instead of having multiple arguments?
pub fn mpv(path: &str, paused: bool) -> MpvResult<MpvHandle> {
    let mut mpv = libmpv::Handle::new().context("creating handle")?;

    mpv.request_log_messages(libmpv::LogLevel::Info)
        .context("setting log level")?;

    let conf_dir = config::mpv_conf_dir();
    log::info!("Using mpv config at: {}", conf_dir.display());
    mpv.set_config_dir(conf_dir).context("setting conf dir")?;

    mpv.read_config_file().context("set reading config file")?;

    let mut mpv = mpv.init().context("initializing")?;

    let version = mpv.version().synch().context("getting version")?;
    log::debug!("mpv version: {version}");

    mpv.set_paused(paused)
        .asynch(DEF_USR)
        .context("setting paused")?;

    mpv.loadfile(path)
        .asynch(DEF_USR)
        .context("loading the file")?;

    mpv.set_idle(libmpv::Idle::No)
        .asynch(DEF_USR)
        .context("setting idle")?;

    Ok(MpvHandle {
        handle: mpv.into_async(),
        state: MpvState::Load,
        // TODO: make the languages configurable
        auto_lang: AutoLang::new(HumanLang::English, HumanLang::Japanese),
    })
}

fn observe_properties(mpv: &mut libmpv::Handle<libmpv::Async>) -> MpvResult<()> {
    mpv.observe_media_title().context("observe media title")?;
    mpv.observe_paused().context("observe paused")?;
    mpv.observe_playback_time()
        .context("observe playback time")?;
    mpv.observe_duration().context("observe duration")?;
    mpv.observe_volume().context("observe volume")?;
    mpv.observe_chapter().context("observe chapter")?;
    mpv.observe_chapters().context("observe chapters")?;
    mpv.observe_track_list().context("observe track list")?;
    Ok(())
}

fn replace<T>(place: &mut T, new: T) -> bool
where
    T: PartialEq,
{
    let are_different = new != *place;
    *place = new;
    are_different
}

fn replace_secs(place: &mut f64, new: f64) -> bool {
    let old = mem::replace(place, new);
    old.trunc() != new.trunc()
}

impl State {
    fn update_state(&mut self, propvalue: &PropertyValue) -> bool {
        match propvalue {
            PropertyValue::Pause(new) => replace(&mut self.pause, *new),
            PropertyValue::MediaTitle(new) => replace(&mut self.title, new.to_string()),
            PropertyValue::PlaybackTime(new) => {
                replace_secs(&mut self.playback_time, *new)
            }
            PropertyValue::Duration(new) => replace_secs(&mut self.duration, *new),
            PropertyValue::Volume(new) => replace(&mut self.volume, *new),
            PropertyValue::Chapters(new) => replace(&mut self.chapters, *new),
            PropertyValue::Chapter(new) => replace(&mut self.chapter, *new),
            PropertyValue::TrackList(new) => {
                replace(&mut self.tracks, node_to_tracks(new))
            }
            _ => false,
        }
    }
}

fn mpv_log_level_convert(level: LogLevel) -> Option<log::Level> {
    match level {
        LogLevel::None => None,
        LogLevel::Fatal => Some(log::Level::Error),
        LogLevel::Error => Some(log::Level::Error),
        LogLevel::Warn => Some(log::Level::Warn),
        LogLevel::Info => Some(log::Level::Info),
        LogLevel::V => Some(log::Level::Debug),
        LogLevel::Debug => Some(log::Level::Debug),
        LogLevel::Trace => Some(log::Level::Trace),
        LogLevel::Unknown(_) => None,
    }
}

fn node_to_tracks(node: &libmpv::Node) -> Vec<Track> {
    let mut tracks = Vec::new();

    if let libmpv::Node::Array(nodes) = node {
        for node in nodes {
            if let libmpv::Node::Map(map) = node {
                let Some(id) = map.get("id").and_then(|id| id.try_to_i64()) else {
                    continue;
                };

                let Some(ttype) = map
                    .get("type")
                    .and_then(|ttype| ttype.try_to_string())
                    .and_then(|ttype| match ttype {
                        "audio" => Some(TrackType::Audio),
                        "video" => Some(TrackType::Video),
                        "sub" => Some(TrackType::Sub),
                        _ => None,
                    })
                else {
                    continue;
                };

                let Some(selected) = map
                    .get("selected")
                    .and_then(|selected| selected.try_to_flag())
                else {
                    continue;
                };

                let title = map
                    .get("title")
                    .and_then(|title| title.try_to_string())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);

                let lang = map
                    .get("lang")
                    .and_then(|lang| lang.try_to_string())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string);

                tracks.push(Track {
                    id,
                    ttype,
                    lang: Lang::new(title, lang),
                    selected,
                });
            }
        }
    }
    tracks
}
