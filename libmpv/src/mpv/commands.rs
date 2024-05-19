use std::{marker::PhantomData, ptr, time::Duration};

use super::{
    macros::{enum_cstr_map, mpv_try, mpv_try_unknown},
    properties::Property,
    Handle, Result,
};
use crate::{bindings::*, see_string::SeeString};

enum_cstr_map! {Command {
    (LoadFile, c"loadfile"),
    (Cycle, c"cycle"),
    (Add, c"add"),
    (Seek, c"seek"),
    (Quit, c"quit"),
}}

impl<T: super::private::HandleState> Handle<T> {
    fn command<'handle, 'args>(
        &'handle mut self,
        command: Command,
        args: impl Into<Vec<SeeString<'args>>>,
    ) -> CmdInner<'handle, 'args> {
        CmdInner {
            ctx: self.ctx,
            command,
            args: args.into(),
            _phantom: PhantomData,
        }
    }
}

struct CmdInner<'handle, 'args> {
    ctx: *mut mpv_handle,
    _phantom: PhantomData<&'handle mut mpv_handle>,
    command: Command,
    args: Vec<SeeString<'args>>,
}

#[must_use = "must actually call it"]
pub struct Cmd<'handle, 'args> {
    inner: Result<CmdInner<'handle, 'args>>,
}

impl<'handle, 'args> Cmd<'handle, 'args> {
    pub fn synch(self) -> Result<()> {
        let inner = self.inner?;
        mpv_try_unknown!(&inner.command)?;
        let mut full_args = Self::prep_args(inner.command, &inner.args);

        mpv_try! {unsafe {mpv_command(inner.ctx, full_args.as_mut_ptr())}}?;
        Ok(())
    }

    pub fn asynch(self, user_data: u64) -> Result<()> {
        let inner = self.inner?;
        mpv_try_unknown!(&inner.command)?;
        let mut full_args = Self::prep_args(inner.command, &inner.args);

        mpv_try! {unsafe {mpv_command_async(inner.ctx, user_data, full_args.as_mut_ptr())}}?;
        Ok(())
    }

    fn prep_args(command: Command, args: &[SeeString<'_>]) -> Vec<*const libc::c_char> {
        let mut full_args = Vec::new();
        full_args.push(command.as_cstr().as_ptr());
        full_args.extend(args.into_iter().map(|a| a.as_ptr()));
        full_args.push(ptr::null());
        full_args
    }
}

impl<'handle, 'args> From<Result<CmdInner<'handle, 'args>>> for Cmd<'handle, 'args> {
    fn from(value: Result<CmdInner<'handle, 'args>>) -> Self {
        Self { inner: value }
    }
}

impl<'handle, 'args> From<CmdInner<'handle, 'args>> for Cmd<'handle, 'args> {
    fn from(value: CmdInner<'handle, 'args>) -> Self {
        Self { inner: Ok(value) }
    }
}

impl<T: super::private::InitState> Handle<T> {
    pub fn loadfile<'a>(&mut self, file: impl Into<SeeString<'a>>) -> Cmd<'_, 'a> {
        let file = file.into();
        // NOTE: filenames are passed as-is to fdopen and the like, mpv does not touch it.
        self.command(Command::LoadFile, [file]).into()
    }

    pub(super) fn cycle(&mut self, prop: Property) -> Cmd<'_, 'static> {
        mpv_try_unknown!(prop)
            .map(|prop| self.command(Command::Cycle, [prop.as_cstr().into()]))
            .into()
    }

    pub(super) fn add_int(&mut self, prop: Property, val: i64) -> Cmd<'_, 'static> {
        mpv_try_unknown!(prop)
            .map(|prop| {
                let val = SeeString::from(val.to_string());
                self.command(Command::Add, [prop.as_cstr().into(), val])
            })
            .into()
    }

    pub(super) fn add_double(&mut self, prop: Property, val: f64) -> Cmd<'_, 'static> {
        mpv_try_unknown!(prop)
            .map(|prop| {
                let val = SeeString::from(val.to_string());
                self.command(Command::Add, [prop.as_cstr().into(), val])
            })
            .into()
    }

    pub fn seek_forward(&mut self, amount: Duration) -> Cmd<'_, 'static> {
        // TODO: does this support fractions of a second?
        let amount = SeeString::from(amount.as_secs().to_string());
        self.command(Command::Seek, [amount]).into()
    }

    pub fn seek_backward(&mut self, amount: Duration) -> Cmd<'_, 'static> {
        // TODO: does this support fractions of a second?
        let amount = -(amount.as_secs() as i64);
        let amount = SeeString::from(amount.to_string());
        self.command(Command::Seek, [amount]).into()
    }

    pub fn seek_abs_percent(&mut self, percent: f64) -> Cmd<'_, 'static> {
        let percent = percent
            .is_nan()
            .then_some(0.0)
            .unwrap_or(percent)
            .clamp(0.0, 100.0);
        let percent = SeeString::from(percent.to_string());
        self.command(Command::Seek, [percent, c"absolute-percent".into()])
            .into()
    }

    pub fn quit(&mut self) -> Cmd<'_, 'static> {
        self.command(Command::Quit, []).into()
    }
}
