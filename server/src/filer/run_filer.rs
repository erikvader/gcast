use std::{thread, time::Duration};

use super::{CacheRcv, SearchRcv, StateSnd};

pub(super) fn run_filer(h_rx: SearchRcv, s_tx: StateSnd, c_rx: CacheRcv) {
    loop {
        log::debug!("run_filer");
        thread::sleep(Duration::from_secs(1));
    }
}
