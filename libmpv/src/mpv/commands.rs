use std::{ffi::CStr, ptr, time::Duration};

use super::{
    macros::{enum_cstr_map, mpv_try, mpv_try_unknown},
    Handle, Init, Result,
};
use crate::{bindings::*, see_string::SeeString, Property};

enum_cstr_map! {Command {
    (LoadFile, c"loadfile"),
    (Cycle, c"cycle"),
    (Add, c"add"),
    (Seek, c"seek"),
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
}

impl Handle<Init> {
    /// returns immediately
    pub fn loadfile<'a>(&mut self, file: impl Into<SeeString<'a>>) -> Result<()> {
        let file = file.into();
        // filenames are passed as-is to fdopen and the like, mpv does not touch it.
        unsafe { self.command_ptr(Command::LoadFile, [file.as_ptr()]) }
    }

    pub(super) fn cycle(&mut self, prop: Property) -> Result<()> {
        mpv_try_unknown!(prop)?;
        self.command(Command::Cycle, [prop.into()])
    }

    pub(super) fn add_int(&mut self, prop: Property, val: i64) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let val = SeeString::from(val);
        self.command(Command::Add, [prop.into(), &val])
    }

    pub(super) fn add_double(&mut self, prop: Property, val: f64) -> Result<()> {
        mpv_try_unknown!(prop)?;
        let val = SeeString::from(val);
        self.command(Command::Add, [prop.into(), &val])
    }

    pub fn seek_forward(&mut self, amount: Duration) -> Result<()> {
        // TODO: does this support fractions of a second?
        let amount = SeeString::from(amount.as_secs());
        self.command(Command::Seek, [&amount])
    }

    pub fn seek_backward(&mut self, amount: Duration) -> Result<()> {
        // TODO: does this support fractions of a second?
        let amount = SeeString::from(-(amount.as_secs() as i64));
        self.command(Command::Seek, [&amount])
    }
}
