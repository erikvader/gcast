use std::{thread, time::Duration};

use crate::repeatable_oneshot::multiplex::MultiplexReceiver;

use super::StateSnd;

pub(super) fn run_filer(rx: MultiplexReceiver<String, ()>, tx: StateSnd) {
    loop {
        log::debug!("run_filer");
        thread::sleep(Duration::from_secs(1));
    }
}
