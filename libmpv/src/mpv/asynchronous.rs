use super::{events::Event, Handle, Sync};
use crate::bindings::*;
use std::{marker::PhantomPinned, pin::Pin, ptr};
use tokio::sync::Notify;

/// Supports everything that sync does, but also some rust async functions
pub struct Async {
    data: Pin<Box<WakeupData>>,
}

unsafe impl Send for Handle<Async> {}

impl super::private::HandleState for Async {
    fn destroy(&mut self, handle: *mut mpv_handle) {
        unsafe {
            mpv_set_wakeup_callback(handle, None, ptr::null_mut());
        }
        // SAFETY: The box will be dropped after here somewhere. Invoking the callback and
        // setting it is done behind a mutex in mpv, so it is not possible that the
        // callback is executing at this point. So it is safe to free the data, it's not
        // in use.
    }
}
impl super::private::InitState for Async {}

// NOTE: doesn't need to be repr(C) since this is never used by C code
struct WakeupData {
    notify: Notify,
    _pin: PhantomPinned,
}

// TODO: something about panics and ffi is UB. Understand the solution and fix?
// https://doc.rust-lang.org/beta/unstable-book/language-features/c-unwind.html
unsafe extern "C" fn wakeup(data: *mut libc::c_void) {
    let Some(data) = (data as *const WakeupData).as_ref() else {
        return;
    };
    data.notify.notify_one();
}

impl Handle<Sync> {
    pub fn into_async(mut self) -> Handle<Async> {
        let data = Box::pin(WakeupData {
            notify: Notify::new(),
            _pin: PhantomPinned,
        });
        let asy = Async { data };
        asy.register(self.ctx);

        Handle {
            ctx: self.disarm(),
            state: asy,
        }
    }
}

impl Async {
    fn register(&self, ctx: *mut mpv_handle) {
        unsafe {
            mpv_set_wakeup_callback(
                ctx,
                Some(wakeup),
                &*self.data as *const WakeupData as *mut libc::c_void,
            );
        }
    }
}

impl Handle<Async> {
    /// Cancel safe since `notified()` is.
    pub async fn wait_event_async(&mut self) -> Event {
        loop {
            match self.wait_event_poll() {
                Event::None => self.state.data.notify.notified().await,
                event => break event,
            }
        }
    }
}
