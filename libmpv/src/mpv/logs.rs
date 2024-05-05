use super::{
    macros::{enum_int_map, mpv_try, mpv_try_unknown},
    Handle, Result,
};
use crate::bindings::*;

enum_int_map! {LogLevel (mpv_log_level) {
    (None, MPV_LOG_LEVEL_NONE),
    (Fatal, MPV_LOG_LEVEL_FATAL),
    (Error, MPV_LOG_LEVEL_ERROR),
    (Warn, MPV_LOG_LEVEL_WARN),
    (Info, MPV_LOG_LEVEL_INFO),
    (V, MPV_LOG_LEVEL_V),
    (Debug, MPV_LOG_LEVEL_DEBUG),
    (Trace, MPV_LOG_LEVEL_TRACE),
}}

impl<T: super::private::InitState> Handle<T> {
    pub fn request_log_messages(&mut self, level: LogLevel) -> Result<()> {
        mpv_try_unknown!(level)?;
        let level_string = match level {
            LogLevel::None => c"no",
            LogLevel::Fatal => c"fatal",
            LogLevel::Error => c"error",
            LogLevel::Warn => c"warn",
            LogLevel::Info => c"info",
            LogLevel::V => c"v",
            LogLevel::Debug => c"debug",
            LogLevel::Trace => c"trace",
            LogLevel::Unknown(_) => unreachable!(),
        };
        mpv_try! {unsafe {mpv_request_log_messages(self.ctx, level_string.as_ptr())}}?;
        Ok(())
    }
}
