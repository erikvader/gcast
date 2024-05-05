use std::{
    ffi::{CStr, CString},
    path::PathBuf,
    ptr,
};

use self::private::Command;

use super::{
    macros::{enum_cstr_map, mpv_try, mpv_try_unknown},
    Handle, Init, Result,
};
use crate::bindings::*;

mod private {
    use super::*;

    enum_cstr_map! {Command {
        (LoadFile, c"loadfile"),
    }}
}

impl<T: super::private::InitState> Handle<T> {
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

impl Handle<Init> {
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
}
