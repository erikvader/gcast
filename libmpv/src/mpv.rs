use std::marker::PhantomData;

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
    pub trait InitState: HandleState {}

    /// A unique owning pointer to a value that behaves as if it were an instance of T.
    /// Inspired by: https://doc.rust-lang.org/src/core/ptr/unique.rs.html
    /// A pointer that is not aliased is safe to be Send/Sync.
    pub struct Unique<T: ?Sized> {
        inner: *mut T,
    }

    unsafe impl<T: Send + ?Sized> Send for Unique<T> {}
    unsafe impl<T: Sync + ?Sized> Sync for Unique<T> {}

    impl<T: ?Sized> Unique<T> {
        /// Safety: No one else should have a copy of this pointer
        pub unsafe fn new(ptr: *mut T) -> Self {
            assert!(!ptr.is_null());
            Self { inner: ptr }
        }

        // TODO: this gives out aliases to this pointer... But it is nicer than having to
        // manually impl Send for each state on the handle, it is handled automatically
        // now. But each access to this pointer should be handled through this wrapper
        // struct to ensure non-aliasing, but there are many places to update...
        pub fn ptr(&mut self) -> *mut T {
            self.inner
        }
    }
}

/// Is initialized and supports everything except for rust async functions
pub struct Sync {
    _priv: (),
}

/// Is not initialized and only supports things that can be done in this early stage
pub struct Uninit {
    _priv: (),
    // NOTE: The mpv docs says that an uninitialized handle probably isn't thread safe
    _not_send_sync: PhantomData<*const ()>,
}

impl private::HandleState for Sync {}
impl private::InitState for Sync {}
impl private::HandleState for Uninit {}

pub struct Handle<T: private::HandleState> {
    ctx: Option<private::Unique<mpv_handle>>,
    state: T,
}

impl Handle<Uninit> {
    pub fn new() -> Result<Handle<Uninit>> {
        if let Some(oldversion) = meets_required_mpv_api_version() {
            return Err(Error::LibMpvTooOld(oldversion));
        }
        let ctx = mpv_try_null! {unsafe { mpv_create() }}?;
        Ok(Handle {
            ctx: Some(unsafe { private::Unique::new(ctx) }),
            state: Uninit {
                _priv: (),
                _not_send_sync: PhantomData,
            },
        })
    }

    pub fn init(mut self) -> Result<Handle<Sync>> {
        mpv_try! {unsafe { mpv_initialize(self.ctx()) }}?;
        let handle = self.transition(Sync { _priv: () });
        // TODO: add a check to make sure the version is at least 0.37.0
        // TODO: the mpv-version property can return git hashes and stuff, so it is not so easy...
        Ok(handle)
    }
}

impl<T: private::HandleState> Drop for Handle<T> {
    fn drop(&mut self) {
        if let Some(mut ctx) = self.ctx.take() {
            let ctx = ctx.ptr();
            self.state.destroy(ctx);
            unsafe { mpv_destroy(ctx) };
        }
    }
}

impl<T: private::InitState> Handle<T> {
    pub fn create_client(&mut self) -> Result<Handle<Sync>> {
        let new_ctx =
            mpv_try_null! {unsafe{mpv_create_client(self.ctx(), std::ptr::null())}}?;
        Ok(Handle {
            ctx: Some(unsafe { private::Unique::new(new_ctx) }),
            state: Sync { _priv: () },
        })
    }

    /// The same as dropping the handle, but also quits the player for all other handles
    pub fn terminate(mut self) {
        let ctx = self.ctx();
        self.state.destroy(ctx);
        unsafe { mpv_terminate_destroy(ctx) };
        self.ctx.take();
    }
}

impl<T: private::HandleState> Handle<T> {
    // TODO: This should not give out copies of this non-aliased pointer...
    fn ctx(&mut self) -> *mut mpv_handle {
        self.ctx.as_mut().expect("must be armed").ptr()
    }

    fn transition<N: private::HandleState>(mut self, new_state: N) -> Handle<N> {
        let ctx = self.ctx();
        self.state.destroy(ctx);
        Handle {
            ctx: self.ctx.take(),
            state: new_state,
        }
    }
}
