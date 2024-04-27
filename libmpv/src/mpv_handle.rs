use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    ptr,
};
use YesNo::*;

use crate::bindings::*;

#[derive(thiserror::Error, Debug)]
pub enum MpvError {
    #[error("Some mpv library function returned NULL")]
    NullPtr,
    #[error("mpv error: {0}")]
    ErrorCode(ErrorCode),
    #[error("the linked against mpv is too old {0} < {}", MINIMUM_MPV_API_VERSION)]
    LibMpvTooOld(libc::c_ulong),
}

pub type Result<T> = std::result::Result<T, MpvError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    EventQueueFull,
    Nomem,
    Uninitialized,
    InvalidParameter,
    OptionNotFound,
    OptionFormat,
    OptionError,
    PropertyNotFound,
    PropertyFormat,
    PropertyUnavailable,
    PropertyError,
    Command,
    LoadingFailed,
    AoInitFailed,
    VoInitFailed,
    NothingToPlay,
    UnknownFormat,
    Unsupported,
    NotImplemented,
    Generic,
    Unknown,
}

const ERROR_CODE_MAP: &[(ErrorCode, mpv_error)] = &[
    (ErrorCode::EventQueueFull, MPV_ERROR_EVENT_QUEUE_FULL),
    (ErrorCode::Nomem, MPV_ERROR_NOMEM),
    (ErrorCode::Uninitialized, MPV_ERROR_UNINITIALIZED),
    (ErrorCode::InvalidParameter, MPV_ERROR_INVALID_PARAMETER),
    (ErrorCode::OptionNotFound, MPV_ERROR_OPTION_NOT_FOUND),
    (ErrorCode::OptionFormat, MPV_ERROR_OPTION_FORMAT),
    (ErrorCode::OptionError, MPV_ERROR_OPTION_ERROR),
    (ErrorCode::PropertyNotFound, MPV_ERROR_PROPERTY_NOT_FOUND),
    (ErrorCode::PropertyFormat, MPV_ERROR_PROPERTY_FORMAT),
    (
        ErrorCode::PropertyUnavailable,
        MPV_ERROR_PROPERTY_UNAVAILABLE,
    ),
    (ErrorCode::PropertyError, MPV_ERROR_PROPERTY_ERROR),
    (ErrorCode::Command, MPV_ERROR_COMMAND),
    (ErrorCode::LoadingFailed, MPV_ERROR_LOADING_FAILED),
    (ErrorCode::AoInitFailed, MPV_ERROR_AO_INIT_FAILED),
    (ErrorCode::VoInitFailed, MPV_ERROR_VO_INIT_FAILED),
    (ErrorCode::NothingToPlay, MPV_ERROR_NOTHING_TO_PLAY),
    (ErrorCode::UnknownFormat, MPV_ERROR_UNKNOWN_FORMAT),
    (ErrorCode::Unsupported, MPV_ERROR_UNSUPPORTED),
    (ErrorCode::NotImplemented, MPV_ERROR_NOT_IMPLEMENTED),
    (ErrorCode::Generic, MPV_ERROR_GENERIC),
];

impl ErrorCode {
    fn from_int(int: mpv_error) -> Option<Self> {
        let common = match int {
            0.. => None,
            _ => Some(Self::Unknown),
        };

        for (rust, c) in ERROR_CODE_MAP {
            if int == *c {
                return Some(*rust);
            }
        }

        common
    }

    fn to_int(self) -> mpv_error {
        for (rust, c) in ERROR_CODE_MAP {
            if self == *rust {
                return *c;
            }
        }
        mpv_error::MIN
    }

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
        match ErrorCode::from_int(int) {
            None => Ok(int),
            Some(err) => Err(MpvError::ErrorCode(err)),
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

    pub fn set_audio_device(&mut self, device: impl Into<String>) -> Result<()> {
        self.set_property_string(Property::AudioDevice, device)
    }
}

impl<T: private::InitState> MpvHandle<T> {
    fn set_property_string(
        &mut self,
        prop: Property,
        value: impl Into<String>,
    ) -> Result<()> {
        let value = CString::new(value.into()).expect("Strings do not contain a null");
        mpv_try! {unsafe { mpv_set_property_string(self.ctx, prop.as_cstr().as_ptr(), value.as_ptr()) }}?;
        Ok(())
    }

    /// The returned property should be UTF-8 except for a few things, see the header
    /// file.
    fn get_property_string(&mut self, prop: Property) -> Result<String> {
        let retval = mpv_try_null! {unsafe { mpv_get_property_string(self.ctx, prop.as_cstr().as_ptr()) }}?;
        let cstr = unsafe { CStr::from_ptr(retval) };
        let rust_str = cstr.to_string_lossy().to_string();
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

enum Property {
    MpvVersion,
    AudioDevice,
    Pause,
}

impl Property {
    fn as_cstr(self) -> &'static CStr {
        match self {
            Property::MpvVersion => c"mpv-version",
            Property::AudioDevice => c"audio-device",
            Property::Pause => c"pause",
        }
    }
}

enum Command {
    LoadFile,
}

impl Command {
    fn as_cstr(self) -> &'static CStr {
        match self {
            Command::LoadFile => c"loadfile",
        }
    }
}

enum YesNo {
    Yes,
    No,
}

impl From<YesNo> for String {
    fn from(value: YesNo) -> Self {
        match value {
            YesNo::Yes => "yes",
            YesNo::No => "no",
        }
        .to_string()
    }
}

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
    #[cfg(unix)]
    pub fn loadfile(&mut self, file: PathBuf) -> Result<()> {
        use std::os::unix::ffi::OsStringExt;
        let file = CString::new(file.into_os_string().into_vec())
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test() -> Result<()> {
        let mut handle = MpvHandle::new()?;
        handle.set_property_string(Property::AudioDevice, "pulse")?;
        let mut handle = handle.init()?;
        let version = handle.get_property_string(Property::MpvVersion)?;
        println!("{}", version);

        handle.pause()?;
        handle.loadurl("https://www.twitch.tv/divvity")?;

        std::thread::sleep(std::time::Duration::from_secs(5));
        handle.play()?;
        std::thread::sleep(std::time::Duration::from_secs(5));

        assert!(false);
        Ok(())
    }
}
