use std::cell::RefCell;

use protocol::{
    to_client::front::{errormsg, filesearch},
    to_server::{errormsgctrl::ErrorMsgCtrl, fscontrol, fsstart, ToServer},
};
use tokio::task::spawn_blocking;

use crate::{
    filer::{cache::Cache, cache_file, read_cache, refresh_cache},
    util::join_handle_wait_take,
};

use super::{Control, MachineResult, StateLogger};

pub(super) async fn filer_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("Filer");
    let mut cache = filer_read_cache_state(ctrl).await?;

    while let Some(msg) = ctrl
        .send_recv_lazy(|| filesearch::Init {
            last_cache_date: cache.updated(),
        })
        .await
    {
        match msg {
            ToServer::FsStart(fsstart::Stop) => break,
            ToServer::FsControl(fscontrol::RefreshCache) => {
                cache = filer_refresh_cache_state(ctrl).await?;
            }
            ToServer::FsControl(fscontrol::Search(search)) => (),
            m => logger.invalid_message(&m),
        }
    }

    Ok(())
}

async fn filer_read_cache_state(ctrl: &mut Control) -> MachineResult<Cache> {
    let _logger = StateLogger::new("FilerReadCache");

    // TODO: flag to say it is initializing/loading
    ctrl.send(filesearch::Init {
        last_cache_date: None,
    })
    .await;

    let cache = read_cache(&cache_file()).await;

    Ok(cache?)
}

async fn filer_refresh_cache_state(ctrl: &mut Control) -> MachineResult<Cache> {
    let logger = StateLogger::new("FilerRefreshCache");

    // "If I were able to define an async closure mutably capturing its environment, it
    // would be possible to invoke the closure multiple times without actually awaiting
    // the future (or dropping it in some other way).
    // This way, we would get multiple Futures with aliased mutable pointers."
    // Source: https://github.com/rust-lang/rust/issues/69446#issuecomment-619354375
    // NOTE: refresh_cache is nice and kind and is always awaiting the closure to
    // completion before calling it again.
    let ctrl = tokio::sync::Mutex::new(ctrl);
    let mut prog_report = |state| async {
        ctrl.try_lock()
            .expect("there shouldn't be two instances locking this at the same time")
            .send(state)
            .await
    };
    let cache = refresh_cache(&mut prog_report).await?;
    let ctrl = ctrl.into_inner();

    while let Some(msg) = ctrl.recv().await {
        match msg {
            ToServer::FsControl(fscontrol::BackToTheBeginning) => break,
            m => logger.invalid_message(&m),
        }
    }

    Ok(cache)
}
