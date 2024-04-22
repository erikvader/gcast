#![allow(non_camel_case_types)]
// /usr/include/mpv/client.h
// https://github.com/mpv-player/mpv/blob/v0.37.0/libmpv/client.h

pub type mpv_error = libc::c_int;
pub const MPV_ERROR_SUCCESS: mpv_error = 0;
pub const MPV_ERROR_EVENT_QUEUE_FULL: mpv_error = -1;
pub const MPV_ERROR_NOMEM: mpv_error = -2;
pub const MPV_ERROR_UNINITIALIZED: mpv_error = -3;
pub const MPV_ERROR_INVALID_PARAMETER: mpv_error = -4;
pub const MPV_ERROR_OPTION_NOT_FOUND: mpv_error = -5;
pub const MPV_ERROR_OPTION_FORMAT: mpv_error = -6;
pub const MPV_ERROR_OPTION_ERROR: mpv_error = -7;
pub const MPV_ERROR_PROPERTY_NOT_FOUND: mpv_error = -8;
pub const MPV_ERROR_PROPERTY_FORMAT: mpv_error = -9;
pub const MPV_ERROR_PROPERTY_UNAVAILABLE: mpv_error = -10;
pub const MPV_ERROR_PROPERTY_ERROR: mpv_error = -11;
pub const MPV_ERROR_COMMAND: mpv_error = -12;
pub const MPV_ERROR_LOADING_FAILED: mpv_error = -13;
pub const MPV_ERROR_AO_INIT_FAILED: mpv_error = -14;
pub const MPV_ERROR_VO_INIT_FAILED: mpv_error = -15;
pub const MPV_ERROR_NOTHING_TO_PLAY: mpv_error = -16;
pub const MPV_ERROR_UNKNOWN_FORMAT: mpv_error = -17;
pub const MPV_ERROR_UNSUPPORTED: mpv_error = -18;
pub const MPV_ERROR_NOT_IMPLEMENTED: mpv_error = -19;
pub const MPV_ERROR_GENERIC: mpv_error = -20;

pub type mpv_format = libc::c_uint;
pub const MPV_FORMAT_NONE: mpv_format = 0;
pub const MPV_FORMAT_STRING: mpv_format = 1;
pub const MPV_FORMAT_OSD_STRING: mpv_format = 2;
pub const MPV_FORMAT_FLAG: mpv_format = 3;
pub const MPV_FORMAT_INT64: mpv_format = 4;
pub const MPV_FORMAT_DOUBLE: mpv_format = 5;
pub const MPV_FORMAT_NODE: mpv_format = 6;
pub const MPV_FORMAT_NODE_ARRAY: mpv_format = 7;
pub const MPV_FORMAT_NODE_MAP: mpv_format = 8;
pub const MPV_FORMAT_BYTE_ARRAY: mpv_format = 9;

#[repr(C)]
pub struct mpv_handle {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct mpv_event {
    pub event_id: mpv_event_id,
    pub error: libc::c_int,
    pub reply_userdata: u64,
    pub data: *mut libc::c_void,
}

pub type mpv_event_id = libc::c_uint;
pub const MPV_EVENT_NONE: mpv_event_id = 0;
pub const MPV_EVENT_SHUTDOWN: mpv_event_id = 1;
pub const MPV_EVENT_LOG_MESSAGE: mpv_event_id = 2;
pub const MPV_EVENT_GET_PROPERTY_REPLY: mpv_event_id = 3;
pub const MPV_EVENT_SET_PROPERTY_REPLY: mpv_event_id = 4;
pub const MPV_EVENT_COMMAND_REPLY: mpv_event_id = 5;
pub const MPV_EVENT_START_FILE: mpv_event_id = 6;
pub const MPV_EVENT_END_FILE: mpv_event_id = 7;
pub const MPV_EVENT_FILE_LOADED: mpv_event_id = 8;
pub const MPV_EVENT_CLIENT_MESSAGE: mpv_event_id = 16;
pub const MPV_EVENT_VIDEO_RECONFIG: mpv_event_id = 17;
pub const MPV_EVENT_AUDIO_RECONFIG: mpv_event_id = 18;
pub const MPV_EVENT_SEEK: mpv_event_id = 20;
pub const MPV_EVENT_PLAYBACK_RESTART: mpv_event_id = 21;
pub const MPV_EVENT_PROPERTY_CHANGE: mpv_event_id = 22;
pub const MPV_EVENT_QUEUE_OVERFLOW: mpv_event_id = 24;
pub const MPV_EVENT_HOOK: mpv_event_id = 25;

extern "C" {
    pub fn mpv_client_api_version() -> libc::c_ulong;
    pub fn mpv_create() -> *mut mpv_handle;
    pub fn mpv_create_client(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
    ) -> *mut mpv_handle;
    pub fn mpv_initialize(ctx: *mut mpv_handle) -> libc::c_int;
    pub fn mpv_destroy(ctx: *mut mpv_handle);
    pub fn mpv_terminate_destroy(ctx: *mut mpv_handle);
    pub fn mpv_free(data: *mut libc::c_void);

    /// the returned string is static
    pub fn mpv_error_string(error: libc::c_int) -> *const libc::c_char;

    pub fn mpv_command(
        ctx: *mut mpv_handle,
        args: *mut *const libc::c_char,
    ) -> libc::c_int;

    pub fn mpv_command_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        args: *mut *const libc::c_char,
    ) -> libc::c_int;

    // pub fn mpv_set_property(
    //     ctx: *mut mpv_handle,
    //     name: *const libc::c_char,
    //     format: mpv_format,
    //     data: *mut libc::c_void,
    // ) -> libc::c_int;

    pub fn mpv_set_property_string(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
        data: *const libc::c_char,
    ) -> libc::c_int;

    pub fn mpv_set_property_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        name: *const libc::c_char,
        format: mpv_format,
        data: *mut libc::c_void,
    ) -> libc::c_int;

    pub fn mpv_get_property_string(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
    ) -> *mut libc::c_char;

    pub fn mpv_get_property_async(
        ctx: *mut mpv_handle,
        reply_userdata: u64,
        name: *const libc::c_char,
        format: mpv_format,
    ) -> libc::c_int;

    pub fn mpv_observe_property(
        mpv: *mut mpv_handle,
        reply_userdata: u64,
        name: *const libc::c_char,
        format: mpv_format,
    ) -> libc::c_int;
    pub fn mpv_wait_event(ctx: *mut mpv_handle, timeout: f64) -> *mut mpv_event;
}

/// Port of the macro MPV_MAKE_VERSION
pub const fn mpv_make_version(
    major: libc::c_ulong,
    minor: libc::c_ulong,
) -> libc::c_ulong {
    (major << 16) | minor
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn versions() {
        assert!(mpv_make_version(2, 1) > mpv_make_version(2, 0));
        assert!(mpv_make_version(2, 1) > mpv_make_version(1, 5));
    }
}
