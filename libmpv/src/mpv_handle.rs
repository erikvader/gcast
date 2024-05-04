use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::ManuallyDrop,
    path::PathBuf,
    ptr,
    time::Duration,
};
use YesNo::*;

use crate::bindings::*;

#[derive(thiserror::Error, Debug)]
pub enum MpvError {
    #[error("Some mpv library function returned NULL")]
    NullPtr,
    #[error("Trying to use an unknown enum variant")]
    Unknown,
    #[error("mpv error: {0}")]
    ErrorCode(ErrorCode),
    #[error("the linked against mpv is too old {0} < {}", MINIMUM_MPV_API_VERSION)]
    LibMpvTooOld(libc::c_ulong),
}

pub type Result<T> = std::result::Result<T, MpvError>;

macro_rules! enum_int_map {
    ($name:ident ($type:ty) {$(($r:ident, $c:ident)),* $(,)*}) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($r),*,
            Unknown($type),
        }

        impl $name {
            pub const fn from_int(int: $type) -> Self {
                match () {
                    $(_ if int == $c => Self::$r),*,
                    _ => Self::Unknown(int),
                }
            }

            pub const fn to_int(self) -> $type {
                match self {
                    $(Self::$r => $c),*,
                    Self::Unknown(int) => int,
                }
            }

            pub const fn is_unknown(self) -> bool {
                matches!(self, Self::Unknown(_))
            }
        }

        impl From<$type> for $name {
            fn from(int: $type) -> Self {
                Self::from_int(int)
            }
        }

        impl From<$name> for $type {
            fn from(e: $name) -> Self {
                e.to_int()
            }
        }
    };
}

macro_rules! enum_cstr_map {
    ($name:ident {$(($r:ident, $c:literal)),* $(,)*}) => {
        #[derive(Debug, Copy, Clone)]
        pub enum $name {
            $($r),*,
            Unknown,
        }

        impl $name {
            pub fn from_cstr(cstr: &CStr) -> Self {
                match () {
                    $(_ if cstr == $c => Self::$r),*,
                    _ => Self::Unknown,
                }
            }

            pub fn from_ptr(ptr: *const libc::c_char) -> Self {
                assert!(!ptr.is_null());
                Self::from_cstr(unsafe{CStr::from_ptr(ptr)})
            }

            pub const fn as_cstr(self) -> &'static CStr {
                match self {
                    $(Self::$r => $c),*,
                    Self::Unknown => c"<UNKNOWN>",
                }
            }

            pub const fn as_ptr(self) -> *const libc::c_char {
                self.as_cstr().as_ptr()
            }

            pub const fn is_unknown(self) -> bool {
                matches!(self, Self::Unknown)
            }
        }

        impl From<&CStr> for $name {
            fn from(int: &CStr) -> Self {
                Self::from_cstr(int)
            }
        }

        impl From<$name> for &'static CStr {
            fn from(e: $name) -> Self {
                e.as_cstr()
            }
        }

        impl AsRef<CStr> for $name {
            fn as_ref(&self) -> &'static CStr {
                self.as_cstr()
            }
        }
    };
}

enum_int_map! {ErrorCode (mpv_error) {
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

macro_rules! mpv_try {
    ($expr:expr) => {{
        let int = ($expr);
        match int {
            0.. => Ok(int),
            _ => Err(MpvError::ErrorCode(ErrorCode::from_int(int))),
        }
    }};
}

macro_rules! mpv_try_null {
    ($expr:expr) => {{
        let ptr = ($expr);
        if ptr.is_null() {
            return Err(MpvError::NullPtr);
        }
        Ok(ptr)
    }};
}

macro_rules! mpv_try_unknown {
    ($expr:expr) => {{
        let val = ($expr);
        if val.is_unknown() {
            return Err(MpvError::Unknown);
        }
        Ok(val)
    }};
}

const MINIMUM_MPV_API_VERSION: libc::c_ulong = mpv_make_version(2, 2);

/// Make sure that the major version of the C api is greater than the minimum supported
/// version
pub fn meets_required_mpv_api_version() -> Option<libc::c_ulong> {
    let version = unsafe { mpv_client_api_version() };
    if version >= MINIMUM_MPV_API_VERSION {
        None
    } else {
        Some(version)
    }
}

mod private {
    pub trait InitState {}
}

pub struct Init;
pub struct Uninit;

impl private::InitState for Init {}
impl private::InitState for Uninit {}

unsafe impl Send for MpvHandle<Init> {}

pub struct MpvHandle<T: private::InitState> {
    ctx: *mut mpv_handle,
    _init: PhantomData<T>,
}

impl MpvHandle<Uninit> {
    pub fn new() -> Result<MpvHandle<Uninit>> {
        if let Some(oldversion) = meets_required_mpv_api_version() {
            return Err(MpvError::LibMpvTooOld(oldversion));
        }
        let ctx = mpv_try_null! {unsafe { mpv_create() }}?;
        Ok(MpvHandle {
            ctx,
            _init: PhantomData,
        })
    }

    pub fn init(self) -> Result<MpvHandle<Init>> {
        mpv_try! {unsafe { mpv_initialize(self.ctx) }}?;
        // Avoid mpv_destroying ctx when self is dropped
        let s = ManuallyDrop::new(self);
        Ok(MpvHandle {
            ctx: s.ctx,
            _init: PhantomData,
        })
        // TODO: add a check to make sure the version is at least 0.37.0
    }

    pub fn set_audio_driver(&mut self, device: AudioDriver) -> Result<()> {
        mpv_try_unknown!(device)?;
        self.set_property_string(Property::AudioDriver, device)
    }
}

impl<T: private::InitState> MpvHandle<T> {
    fn set_property_string(
        &mut self,
        prop: Property,
        value: impl AsRef<CStr>,
    ) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let value = value.as_ref();
        mpv_try! {unsafe { mpv_set_property_string(self.ctx, prop.as_cstr().as_ptr(), value.as_ptr()) }}?;
        Ok(())
    }

    /// The returned property should be UTF-8 except for a few things, see the header
    /// file.
    fn get_property_string(&mut self, prop: Property) -> Result<String> {
        mpv_try_unknown!(prop)?;
        let retval = mpv_try_null! {unsafe { mpv_get_property_string(self.ctx, prop.as_cstr().as_ptr()) }}?;
        let cstr = unsafe { CStr::from_ptr(retval) };
        let rust_str = cstr.to_string_lossy().into_owned();
        assert_ne!(retval as *const u8, rust_str.as_ptr());
        unsafe { mpv_free(retval as *mut libc::c_void) };
        Ok(rust_str)
    }

    fn command<const N: usize>(
        &mut self,
        command: Command,
        args: [&CStr; N],
    ) -> Result<()> {
        unsafe { self.command_ptr(command, args.map(CStr::as_ptr)) }
    }

    unsafe fn command_ptr<const N: usize>(
        &mut self,
        command: Command,
        args: [*const libc::c_char; N],
    ) -> Result<()> {
        mpv_try_unknown!(command)?;
        // TODO: can't use full_args = [ptr::null; {N+2}] yet
        let mut full_args = Vec::new();
        full_args.push(command.as_cstr().as_ptr());
        full_args.extend(args);
        full_args.push(ptr::null());

        mpv_try! {mpv_command(self.ctx, full_args.as_mut_ptr())}?;
        Ok(())
    }
}

impl<T: private::InitState> Drop for MpvHandle<T> {
    fn drop(&mut self) {
        unsafe { mpv_destroy(self.ctx) };
    }
}

enum_cstr_map! {Property {
    (MpvVersion, c"mpv-version"),
    (AudioDriver, c"ao"),
    (Pause, c"pause"),
    (InputDefaultBindings, c"input-default-bindings"),
    (InputVoKeyboard, c"input-vo-keyboard"),
}}

enum_cstr_map! {Command {
    (LoadFile, c"loadfile"),
}}

enum_cstr_map! {YesNo {
    (Yes, c"yes"),
    (No, c"no"),
}}

enum_cstr_map! {AudioDriver {
    (Pulse, c"pulse"),
}}

impl MpvHandle<Init> {
    pub fn create_client(&mut self) -> Result<MpvHandle<Init>> {
        let ctx = mpv_try_null! {unsafe{mpv_create_client(self.ctx, ptr::null())}}?;
        Ok(MpvHandle {
            ctx,
            _init: PhantomData,
        })
    }

    pub fn terminate(self) {
        // Avoid mpv_destroying ctx when self is dropped
        let s = ManuallyDrop::new(self);
        unsafe { mpv_terminate_destroy(s.ctx) };
    }

    /// returns immediately
    // NOTE: needs a pathbuf because a null-terminated string needs to be allocated anyway
    #[cfg(unix)]
    pub fn loadfile(&mut self, file: impl Into<PathBuf>) -> Result<()> {
        use std::os::unix::ffi::OsStringExt;
        let file = CString::new(file.into().into_os_string().into_vec())
            .expect("PathBuf does not contain a null");

        // filenames are passed as-is to fdopen and the like, mpv does not touch it.
        unsafe { self.command_ptr(Command::LoadFile, [file.as_ptr()]) }
    }

    // TODO: URL type
    pub fn loadurl(&mut self, url: impl Into<String>) -> Result<()> {
        let url = CString::new(url.into()).expect("Strings do not contain a null");
        self.command(Command::LoadFile, [&url])
    }

    pub fn version(&mut self) -> Result<String> {
        self.get_property_string(Property::MpvVersion)
    }

    pub fn pause(&mut self) -> Result<()> {
        self.set_property_string(Property::Pause, Yes)
    }

    pub fn play(&mut self) -> Result<()> {
        self.set_property_string(Property::Pause, No)
    }

    pub fn enable_default_bindings(&mut self) -> Result<()> {
        self.set_property_string(Property::InputDefaultBindings, Yes)?;
        self.set_property_string(Property::InputVoKeyboard, Yes)?;
        Ok(())
    }

    pub fn observe_property(&mut self, prop: Property, format: Format) -> Result<()> {
        mpv_try_unknown!(prop)?;
        mpv_try_unknown!(format)?;
        mpv_try!(unsafe {
            mpv_observe_property(self.ctx, 0, prop.as_ptr(), format.to_int())
        })?;
        Ok(())
    }

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
            EventID::LogMessage => todo!(),
            EventID::StartFile => Event::StartFile,
            EventID::EndFile => Event::EndFile {
                // TODO:
                reason: EndReason::Unknown(0),
                error: None,
            },
            EventID::FileLoaded => Event::FileLoaded,
            EventID::PropertyChange => {
                let data = (*event).data;
                assert!(!data.is_null());
                let data = data as *const mpv_event_property;
                match (
                    Property::from_ptr((*data).name),
                    Format::from_int((*data).format),
                ) {
                    (property, Format::String) if !property.is_unknown() => {
                        let value = (*data).data as *const *const libc::c_char;
                        assert!(!value.is_null());
                        let value = *value;
                        // NOTE: From the doc of MPV_FORMAT_STRING: although the encoding
                        // is usually UTF-8, this is not always the case. File tags often
                        // store strings in some legacy codepage, and even filenames don't
                        // necessarily have to be in UTF-8 (at least on Linux).
                        let value = CStr::from_ptr(value).to_string_lossy().into_owned();
                        Event::PropertyChange {
                            property,
                            value: PropertyValue::String(value),
                        }
                    }
                    (property, Format::None) => Event::PropertyChange {
                        property,
                        value: PropertyValue::None,
                    },
                    (property, format) => Event::PropertyChange {
                        property,
                        value: PropertyValue::Unsupported(format),
                    },
                }
            }
            EventID::QueueOverflow => Event::QueueOverflow,
            EventID::Unknown(id) => Event::UnsupportedEvent(id.into()),
        }
    }
}

enum_int_map! {EventID (mpv_event_id) {
    (None, MPV_EVENT_NONE),
    (Shutdown, MPV_EVENT_SHUTDOWN),
    (LogMessage, MPV_EVENT_LOG_MESSAGE),
    (StartFile, MPV_EVENT_START_FILE),
    (EndFile, MPV_EVENT_END_FILE),
    (FileLoaded, MPV_EVENT_FILE_LOADED),
    (PropertyChange, MPV_EVENT_PROPERTY_CHANGE),
    (QueueOverflow, MPV_EVENT_QUEUE_OVERFLOW),
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
    PropertyChange {
        property: Property,
        value: PropertyValue,
    },
    StartFile,
    EndFile {
        reason: EndReason,
        error: Option<ErrorCode>,
    },
    FileLoaded,
    UnsupportedEvent(EventID),
}

#[derive(Debug)]
pub enum PropertyValue {
    String(String),
    None,
    Unsupported(Format),
}

enum_int_map! {EndReason (mpv_end_file_reason) {
    (EOF, MPV_END_FILE_REASON_EOF),
    (Stop, MPV_END_FILE_REASON_STOP),
    (Quit, MPV_END_FILE_REASON_QUIT),
    (Error, MPV_END_FILE_REASON_ERROR),
    (Redirect, MPV_END_FILE_REASON_REDIRECT),
}}

// TODO: make private?
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

enum_int_map! {Format (mpv_format) {
    (None, MPV_FORMAT_NONE),
    (String, MPV_FORMAT_STRING),
    (OsdString, MPV_FORMAT_OSD_STRING),
    (Flag, MPV_FORMAT_FLAG),
    (Int64, MPV_FORMAT_INT64),
    (Double, MPV_FORMAT_DOUBLE),
    (Node, MPV_FORMAT_NODE),
    (NodeArray, MPV_FORMAT_NODE_ARRAY),
    (NodeMap, MPV_FORMAT_NODE_MAP),
    (ByteArray, MPV_FORMAT_BYTE_ARRAY),
}}
