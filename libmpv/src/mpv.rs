use std::{marker::PhantomData, mem::ManuallyDrop, ptr};

use crate::bindings::*;

use self::{
    error::Error,
    macros::{mpv_try, mpv_try_null},
};

pub mod commands;
pub mod data;
pub mod error;
pub mod events;
pub mod logs;
mod macros;
pub mod properties;

pub type Result<T> = std::result::Result<T, Error>;
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

unsafe impl Send for Handle<Init> {}

pub struct Handle<T: private::InitState> {
    ctx: *mut mpv_handle,
    _init: PhantomData<T>,
}

impl Handle<Uninit> {
    pub fn new() -> Result<Handle<Uninit>> {
        if let Some(oldversion) = meets_required_mpv_api_version() {
            return Err(Error::LibMpvTooOld(oldversion));
        }
        let ctx = mpv_try_null! {unsafe { mpv_create() }}?;
        Ok(Handle {
            ctx,
            _init: PhantomData,
        })
    }

    pub fn init(self) -> Result<Handle<Init>> {
        mpv_try! {unsafe { mpv_initialize(self.ctx) }}?;
        // NOTE: Avoid mpv_destroying ctx when self is dropped
        let s = ManuallyDrop::new(self);
        let handle = Handle {
            ctx: s.ctx,
            _init: PhantomData,
        };
        // TODO: add a check to make sure the version is at least 0.37.0
        // TODO: the mpv-version property can return git hashes and stuff, so it is not so easy...
        Ok(handle)
    }
}

impl<T: private::InitState> Drop for Handle<T> {
    fn drop(&mut self) {
        unsafe { mpv_destroy(self.ctx) };
    }
}

impl Handle<Init> {
    pub fn create_client(&mut self) -> Result<Handle<Init>> {
        let ctx = mpv_try_null! {unsafe{mpv_create_client(self.ctx, ptr::null())}}?;
        Ok(Handle {
            ctx,
            _init: PhantomData,
        })
    }

    pub fn terminate(self) {
        // Avoid mpv_destroying ctx when self is dropped
        let s = ManuallyDrop::new(self);
        unsafe { mpv_terminate_destroy(s.ctx) };
    }
}
