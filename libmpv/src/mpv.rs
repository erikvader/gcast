use std::ptr;

use crate::bindings::*;

use self::{
    error::Error,
    macros::{mpv_try, mpv_try_null},
};

pub mod asynchronous;
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
    use super::mpv_handle;

    /// Any valid state for a handle
    pub trait HandleState {
        fn destroy(&mut self, _handle: *mut mpv_handle) {}
    }

    /// A state that is initialized
    pub trait Init: HandleState {}
}

/// Is initialized and supports everything except for rust async functions
pub struct Sync;

/// Is not initialized and only supports things that can be done in this early stage
pub struct Uninit;

impl private::HandleState for Sync {}
impl private::Init for Sync {}
impl private::HandleState for Uninit {}

unsafe impl Send for Handle<Sync> {}

pub struct Handle<T: private::HandleState> {
    ctx: *mut mpv_handle,
    state: T,
}

impl Handle<Uninit> {
    pub fn new() -> Result<Handle<Uninit>> {
        if let Some(oldversion) = meets_required_mpv_api_version() {
            return Err(Error::LibMpvTooOld(oldversion));
        }
        let ctx = mpv_try_null! {unsafe { mpv_create() }}?;
        Ok(Handle { ctx, state: Uninit })
    }

    pub fn init(mut self) -> Result<Handle<Sync>> {
        mpv_try! {unsafe { mpv_initialize(self.ctx) }}?;
        let handle = Handle {
            ctx: self.disarm(),
            state: Sync,
        };
        // TODO: add a check to make sure the version is at least 0.37.0
        // TODO: the mpv-version property can return git hashes and stuff, so it is not so easy...
        Ok(handle)
    }
}

impl<T: private::HandleState> Drop for Handle<T> {
    fn drop(&mut self) {
        if self.is_armed() {
            self.state.destroy(self.ctx);
            unsafe { mpv_destroy(self.ctx) };
            self.disarm();
        }
    }
}

impl<T: private::Init> Handle<T> {
    pub fn create_client(&mut self) -> Result<Handle<Sync>> {
        let ctx = mpv_try_null! {unsafe{mpv_create_client(self.ctx, ptr::null())}}?;
        Ok(Handle { ctx, state: Sync })
    }

    /// The same as dropping the handle, but also quits the player for all other handles
    pub fn terminate(mut self) {
        assert!(self.is_armed());
        self.state.destroy(self.ctx);
        unsafe { mpv_terminate_destroy(self.ctx) };
        self.disarm();
    }
}

impl<T: private::HandleState> Handle<T> {
    fn disarm(&mut self) -> *mut mpv_handle {
        assert!(self.is_armed());
        let ctx = self.ctx;
        self.ctx = ptr::null::<mpv_handle>() as *mut mpv_handle;
        ctx
    }

    fn is_armed(&self) -> bool {
        !self.ctx.is_null()
    }
}
