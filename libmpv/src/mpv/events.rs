use std::time::Duration;

use super::{
    data::Format,
    error::ErrorCode,
    logs::LogLevel,
    macros::enum_int_map,
    properties::{Property, PropertyValue},
    Handle, Init,
};

use crate::{
    bindings::*,
    mpv::data::{ptr_to_node, ptr_to_string},
};

enum_int_map! {pub EventID (mpv_event_id) {
    (None, MPV_EVENT_NONE),
    (Shutdown, MPV_EVENT_SHUTDOWN),
    (LogMessage, MPV_EVENT_LOG_MESSAGE),
    (GetPropertyReply, MPV_EVENT_GET_PROPERTY_REPLY),
    (SetPropertyReply, MPV_EVENT_SET_PROPERTY_REPLY),
    (CommandReply, MPV_EVENT_COMMAND_REPLY),
    (StartFile, MPV_EVENT_START_FILE),
    (EndFile, MPV_EVENT_END_FILE),
    (FileLoaded, MPV_EVENT_FILE_LOADED),
    (ClientMessage, MPV_EVENT_CLIENT_MESSAGE),
    (VideoReconfig, MPV_EVENT_VIDEO_RECONFIG),
    (AudioReconfig, MPV_EVENT_AUDIO_RECONFIG),
    (Seek, MPV_EVENT_SEEK),
    (PlaybackRestart, MPV_EVENT_PLAYBACK_RESTART),
    (PropertyChange, MPV_EVENT_PROPERTY_CHANGE),
    (QueueOverflow, MPV_EVENT_QUEUE_OVERFLOW),
    (Hook, MPV_EVENT_HOOK),
}}

#[derive(Debug)]
pub enum Event {
    None,
    Shutdown,
    Log {
        prefix: String,
        level: LogLevel,
        text: String,
    },
    QueueOverflow,
    PropertyChange(PropertyValue),
    PropertyChangeError(Format, Property),
    StartFile,
    EndFile {
        reason: EndReason,
        error: Option<ErrorCode>,
    },
    FileLoaded,
    UnsupportedEvent(EventID),
}

enum_int_map! {pub EndReason (mpv_end_file_reason) {
    (EOF, MPV_END_FILE_REASON_EOF),
    (Stop, MPV_END_FILE_REASON_STOP),
    (Quit, MPV_END_FILE_REASON_QUIT),
    (Error, MPV_END_FILE_REASON_ERROR),
    (Redirect, MPV_END_FILE_REASON_REDIRECT),
}}

impl Handle<Init> {
    pub fn wait_event(&mut self, timeout: Duration) -> Event {
        unsafe { self.wait_event_raw(timeout.as_secs_f64()) }
    }

    pub fn wait_event_infinite(&mut self) -> Event {
        unsafe { self.wait_event_raw(-1.0) }
    }

    unsafe fn wait_event_raw(&mut self, timeout: f64) -> Event {
        let event = unsafe { mpv_wait_event(self.ctx, timeout) };
        assert!(!event.is_null(), "is never null");
        match EventID::from_int((*event).event_id) {
            EventID::None => Event::None,
            EventID::Shutdown => Event::Shutdown,
            EventID::StartFile => Event::StartFile,
            EventID::EndFile => {
                let data = (*event).data;
                assert!(!data.is_null());
                let data = data as *const mpv_event_end_file;
                let reason = EndReason::from_int((*data).reason);
                let error = if matches!(reason, EndReason::Error) {
                    Some(ErrorCode::from_int((*data).error))
                } else {
                    None
                };
                Event::EndFile { reason, error }
            }
            EventID::FileLoaded => Event::FileLoaded,
            EventID::PropertyChange => {
                let data = (*event).data;
                assert!(!data.is_null());
                let data = data as *const mpv_event_property;
                let property = Property::from_ptr((*data).name);
                let format = Format::from_int((*data).format);

                let property_value = match format {
                    Format::String => {
                        let value = (*data).data as *const *const libc::c_char;
                        assert!(!value.is_null());
                        let value = *value;
                        let value = ptr_to_string(value);
                        match property {
                            Property::MediaTitle => {
                                Some(PropertyValue::MediaTitle(value))
                            }
                            _ => None,
                        }
                    }
                    Format::Flag => {
                        let value = (*data).data as *const libc::c_int;
                        assert!(!value.is_null());
                        let value = *value;
                        let value = value != 0;
                        match property {
                            Property::Pause => Some(PropertyValue::Pause(value)),
                            _ => None,
                        }
                    }
                    Format::Int64 => {
                        let value = (*data).data as *const i64; // int64_t
                        assert!(!value.is_null());
                        let value = *value;
                        match property {
                            Property::Chapter => Some(PropertyValue::Chapter(value)),
                            Property::Chapters => Some(PropertyValue::Chapters(value)),
                            _ => None,
                        }
                    }
                    Format::Double => {
                        let value = (*data).data as *const libc::c_double;
                        assert!(!value.is_null());
                        let value = *value;
                        match property {
                            Property::PlaybackTime => {
                                Some(PropertyValue::PlaybackTime(value))
                            }
                            Property::Duration => Some(PropertyValue::Duration(value)),
                            Property::Volume => Some(PropertyValue::Volume(value)),
                            _ => None,
                        }
                    }
                    Format::Node => {
                        let value = (*data).data as *const mpv_node;
                        assert!(!value.is_null());
                        let value = ptr_to_node(value);
                        match property {
                            Property::TrackList => Some(PropertyValue::TrackList(value)),
                            _ => None,
                        }
                    }
                    _ => None,
                };

                if let Some(value) = property_value {
                    Event::PropertyChange(value)
                } else {
                    Event::PropertyChangeError(format, property)
                }
            }
            EventID::QueueOverflow => Event::QueueOverflow,
            EventID::LogMessage => {
                let data = (*event).data;
                assert!(!data.is_null());
                let data = data as *const mpv_event_log_message;
                let prefix = ptr_to_string((*data).prefix);
                let text = ptr_to_string((*data).text);
                let level = LogLevel::from_int((*data).log_level);
                Event::Log {
                    prefix,
                    level,
                    text,
                }
            }
            unsupported => Event::UnsupportedEvent(unsupported),
        }
    }
}
