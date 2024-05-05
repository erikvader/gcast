#![allow(non_camel_case_types)]
// /usr/include/mpv/client.h
// https://github.com/mpv-player/mpv/blob/v0.37.0/libmpv/client.h

pub type int64_t = i64;

pub type mpv_error = libc::c_int;
// pub const MPV_ERROR_SUCCESS: mpv_error = 0;
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

#[repr(C)]
pub struct mpv_event_property {
    pub name: *const libc::c_char,
    pub format: mpv_format,
    pub data: *mut libc::c_void,
}

#[repr(C)]
pub struct mpv_event_end_file {
    pub reason: mpv_end_file_reason,
    pub error: libc::c_int,
    pub playlist_entry_id: i64,
    pub playlist_insert_id: i64,
    pub playlist_insert_num_entries: libc::c_int,
}

#[repr(C)]
pub struct mpv_event_log_message {
    pub prefix: *const libc::c_char,
    pub level: *const libc::c_char,
    pub text: *const libc::c_char,
    pub log_level: mpv_log_level,
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

pub type mpv_log_level = libc::c_uint;
/// "no"    - disable absolutely all messages
pub const MPV_LOG_LEVEL_NONE: mpv_log_level = 0;
/// "fatal" - critical/aborting errors
pub const MPV_LOG_LEVEL_FATAL: mpv_log_level = 10;
/// "error" - simple errors
pub const MPV_LOG_LEVEL_ERROR: mpv_log_level = 20;
/// "warn"  - possible problems
pub const MPV_LOG_LEVEL_WARN: mpv_log_level = 30;
/// "info"  - informational message
pub const MPV_LOG_LEVEL_INFO: mpv_log_level = 40;
/// "v"     - noisy informational message
pub const MPV_LOG_LEVEL_V: mpv_log_level = 50;
/// "debug" - very noisy technical information
pub const MPV_LOG_LEVEL_DEBUG: mpv_log_level = 60;
/// "trace" - extremely noisy
pub const MPV_LOG_LEVEL_TRACE: mpv_log_level = 70;

pub type mpv_end_file_reason = libc::c_uint;
pub const MPV_END_FILE_REASON_EOF: mpv_end_file_reason = 0;
pub const MPV_END_FILE_REASON_STOP: mpv_end_file_reason = 2;
pub const MPV_END_FILE_REASON_QUIT: mpv_end_file_reason = 3;
pub const MPV_END_FILE_REASON_ERROR: mpv_end_file_reason = 4;
pub const MPV_END_FILE_REASON_REDIRECT: mpv_end_file_reason = 5;

#[repr(C)]
pub struct mpv_node {
    pub u: mpv_node_u,
    pub format: mpv_format,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union mpv_node_u {
    pub string: *mut libc::c_char,
    pub flag: libc::c_int,
    pub int64: i64,
    pub double: f64,
    pub list: *mut mpv_node_list,
    // pub ba: *mut mpv_byte_array, // NOTE: purposfully ignoring
}

#[repr(C)]
pub struct mpv_node_list {
    pub num: libc::c_int,
    pub values: *mut mpv_node,
    pub keys: *mut *mut libc::c_char,
}

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
    pub fn mpv_free_node_contents(node: *mut mpv_node);

    /// the returned string is static
    pub fn mpv_error_string(error: libc::c_int) -> *const libc::c_char;

    pub fn mpv_command(
        ctx: *mut mpv_handle,
        args: *mut *const libc::c_char,
    ) -> libc::c_int;

    // pub fn mpv_command_async(
    //     ctx: *mut mpv_handle,
    //     reply_userdata: u64,
    //     args: *mut *const libc::c_char,
    // ) -> libc::c_int;

    pub fn mpv_set_property(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
        format: mpv_format,
        data: *mut libc::c_void,
    ) -> libc::c_int;

    pub fn mpv_set_property_string(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
        data: *const libc::c_char,
    ) -> libc::c_int;

    // pub fn mpv_set_property_async(
    //     ctx: *mut mpv_handle,
    //     reply_userdata: u64,
    //     name: *const libc::c_char,
    //     format: mpv_format,
    //     data: *mut libc::c_void,
    // ) -> libc::c_int;

    pub fn mpv_get_property(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
        format: mpv_format,
        data: *mut libc::c_void,
    ) -> libc::c_int;

    pub fn mpv_get_property_string(
        ctx: *mut mpv_handle,
        name: *const libc::c_char,
    ) -> *mut libc::c_char;

    // pub fn mpv_get_property_async(
    //     ctx: *mut mpv_handle,
    //     reply_userdata: u64,
    //     name: *const libc::c_char,
    //     format: mpv_format,
    // ) -> libc::c_int;

    pub fn mpv_observe_property(
        mpv: *mut mpv_handle,
        reply_userdata: u64,
        name: *const libc::c_char,
        format: mpv_format,
    ) -> libc::c_int;
    pub fn mpv_wait_event(ctx: *mut mpv_handle, timeout: f64) -> *mut mpv_event;

    pub fn mpv_request_log_messages(
        ctx: *mut mpv_handle,
        min_level: *const libc::c_char,
    ) -> libc::c_int;
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
