use std::{
    ffi::{CStr, CString},
    ops::Neg,
    path::PathBuf,
    ptr,
    time::Duration,
};

use super::{
    macros::{enum_cstr_map, mpv_try, mpv_try_unknown},
    Handle, Init, Result,
};
use crate::{bindings::*, Property};

enum_cstr_map! {Command {
    (LoadFile, c"loadfile"),
    (Cycle, c"cycle"),
    (Add, c"add"),
    (Seek, c"seek"),
    (CycleValues, c"cycle-values"),
}}

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

    fn command_varargs(&mut self, command: Command, mut args: Vec<&CStr>) -> Result<()> {
        mpv_try_unknown!(command)?;
        args.insert(0, command.as_cstr());
        let mut args: Vec<*const libc::c_char> =
            args.into_iter().map(CStr::as_ptr).collect();
        args.push(ptr::null());

        mpv_try! {unsafe{mpv_command(self.ctx, args.as_mut_ptr())}}?;
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

    pub(super) fn cycle(&mut self, prop: Property) -> Result<()> {
        mpv_try_unknown!(prop)?;
        self.command(Command::Cycle, [prop.into()])
    }

    pub(super) fn add_int(&mut self, prop: Property, val: i64) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let val = CString::new(val.to_string()).expect("numbers don't contain null");
        self.command(Command::Add, [prop.into(), &val])
    }

    pub(super) fn add_double(&mut self, prop: Property, val: f64) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let val = CString::new(val.to_string()).expect("numbers don't contain null");
        self.command(Command::Add, [prop.into(), &val])
    }

    pub fn seek_forward(&mut self, amount: Duration) -> Result<()> {
        // TODO: does this support fractions of a second?
        let amount = CString::new(amount.as_secs().to_string()).expect("no null");
        self.command(Command::Seek, [&amount])
    }

    pub fn seek_backward(&mut self, amount: Duration) -> Result<()> {
        // TODO: does this support fractions of a second?
        let amount =
            CString::new((amount.as_secs() as i64).neg().to_string()).expect("no null");
        self.command(Command::Seek, [&amount])
    }

    pub(super) fn cycle_values<S: Into<String>>(
        &mut self,
        prop: Property,
        values: Vec<S>,
    ) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let values: Vec<_> = values
            .into_iter()
            .map(|s| CString::new(s.into()).expect("no nulls"))
            .collect();

        let mut full_args = vec![prop.as_cstr()];
        full_args.extend(values.iter().map(CString::as_c_str));

        self.command_varargs(Command::CycleValues, full_args)
    }
}
