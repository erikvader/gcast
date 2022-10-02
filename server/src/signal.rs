use std::{mem::MaybeUninit, ptr};

use libc::{c_int, sigaction, SIGINT, SIGTERM};
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
};

// This should be fine if used once. Not sure how tokio will handle installing signal
// handlers again after being restored from the outside.
pub async fn signal_received() {
    log::debug!("Saving and installing signal handlers");
    let _sig_saver = SignalSaver::new();

    let mut sigint = signal(SignalKind::interrupt()).expect("sigint signal failed");
    let mut sigterm = signal(SignalKind::terminate()).expect("sigterm signal failed");
    select! {
        _ = sigint.recv() => {
            log::debug!("Received sigint");
        },
        _ = sigterm.recv() => {
            log::debug!("Received sigterm");
        }
    }
}

struct SignalSaver {
    sigint: sigaction,
    sigterm: sigaction,
}

impl SignalSaver {
    fn new() -> SignalSaver {
        SignalSaver {
            sigint: unsafe { get_sigaction(SIGINT) },
            sigterm: unsafe { get_sigaction(SIGTERM) },
        }
    }

    fn restore(&self) {
        unsafe {
            set_sigaction(SIGINT, &self.sigint);
            set_sigaction(SIGTERM, &self.sigterm);
        }
    }
}

impl Drop for SignalSaver {
    fn drop(&mut self) {
        log::debug!("Resetting signal handlers");
        self.restore();
    }
}

unsafe fn set_sigaction(sig: c_int, act: &sigaction) {
    assert!(sigaction(sig, act, ptr::null_mut()) == 0);
}

unsafe fn get_sigaction(sig: c_int) -> sigaction {
    let mut old = MaybeUninit::zeroed();
    assert!(sigaction(sig, ptr::null(), old.as_mut_ptr()) == 0);
    old.assume_init()
}
