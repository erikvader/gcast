use super::{Handle, Init};
use crate::{bindings::*, Event};
use std::ptr;
use tokio::sync::Notify;

pub struct AsyncHandle {
    handle: Handle<Init>,
    data: Box<WakeupData>,
}

// NOTE: doesn't need to be repr(C) since this is never used by C code
struct WakeupData {
    notify: Notify,
}

// TODO: something about panics and ffi is UB. Understand the solution and fix?
// https://doc.rust-lang.org/beta/unstable-book/language-features/c-unwind.html
unsafe extern "C" fn wakeup(data: *mut libc::c_void) {
    let Some(data) = (data as *const WakeupData).as_ref() else {
        return;
    };
    data.notify.notify_one();
}

impl AsyncHandle {
    pub fn new(handle: Handle<Init>) -> Self {
        let data = Box::new(WakeupData {
            notify: Notify::new(),
        });

        unsafe {
            mpv_set_wakeup_callback(
                handle.ctx,
                Some(wakeup),
                &*data as *const WakeupData as *mut libc::c_void,
            );
        }
        Self { handle, data }
    }
}

impl std::ops::Deref for AsyncHandle {
    type Target = Handle<Init>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl std::ops::DerefMut for AsyncHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handle
    }
}

impl Drop for AsyncHandle {
    fn drop(&mut self) {
        unsafe {
            mpv_set_wakeup_callback(self.handle.ctx, None, ptr::null_mut());
        }
        // SAFETY: The box will be dropped after here somewhere. Invoking the callback and
        // setting it is done behind a mutex in mpv, so it is not possible that the
        // callback is executing at this point. So it is safe to free the data, it's not
        // in use.
    }
}

impl AsyncHandle {
    /// Cancel safe since `notified()` is.
    pub async fn wait_event_async(&mut self) -> Event {
        loop {
            match self.handle.wait_event_poll() {
                Event::None => self.data.notify.notified().await,
                event => break event,
            }
        }
    }
}
