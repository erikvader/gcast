use std::ffi::CStr;

use super::{macros::enum_int_map, MINIMUM_MPV_API_VERSION};
use crate::bindings::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Some mpv library function returned NULL")]
    NullPtr,
    #[error("Trying to use an unknown enum variant")]
    Unknown,
    #[error("mpv error: {0}")]
    ErrorCode(ErrorCode),
    #[error("the linked against mpv is too old {0} < {}", MINIMUM_MPV_API_VERSION)]
    LibMpvTooOld(libc::c_ulong),
}

enum_int_map! {pub ErrorCode (mpv_error) {
    (EventQueueFull, MPV_ERROR_EVENT_QUEUE_FULL),
    (Nomem, MPV_ERROR_NOMEM),
    (Uninitialized, MPV_ERROR_UNINITIALIZED),
    (InvalidParameter, MPV_ERROR_INVALID_PARAMETER),
    (OptionNotFound, MPV_ERROR_OPTION_NOT_FOUND),
    (OptionFormat, MPV_ERROR_OPTION_FORMAT),
    (OptionError, MPV_ERROR_OPTION_ERROR),
    (PropertyNotFound, MPV_ERROR_PROPERTY_NOT_FOUND),
    (PropertyFormat, MPV_ERROR_PROPERTY_FORMAT),
    (PropertyUnavailable, MPV_ERROR_PROPERTY_UNAVAILABLE),
    (PropertyError, MPV_ERROR_PROPERTY_ERROR),
    (Command, MPV_ERROR_COMMAND),
    (LoadingFailed, MPV_ERROR_LOADING_FAILED),
    (AoInitFailed, MPV_ERROR_AO_INIT_FAILED),
    (VoInitFailed, MPV_ERROR_VO_INIT_FAILED),
    (NothingToPlay, MPV_ERROR_NOTHING_TO_PLAY),
    (UnknownFormat, MPV_ERROR_UNKNOWN_FORMAT),
    (Unsupported, MPV_ERROR_UNSUPPORTED),
    (NotImplemented, MPV_ERROR_NOT_IMPLEMENTED),
    (Generic, MPV_ERROR_GENERIC),
}}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        let cstr = unsafe { mpv_error_string(self.to_int()) };
        assert!(!cstr.is_null());
        let cstr = unsafe { CStr::from_ptr(cstr) };
        cstr.to_str().expect("assuming this is UTF-8")
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
