use protocol::{
    to_client::front::filesearch,
    to_server::{fscontrol, fsstart, mpvstart, ToServer},
};

use crate::filer::{self, cache::Cache, cache_file, read_cache, refresh_cache};

use super::{Control, Jump, LockedControl, MachineResult, StateLogger};

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
            ToServer::FsControl(fscontrol::Search(search)) if search.is_empty() => {
                filer_search_state(ctrl, &cache).await?;
            }
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

    let ctrl = LockedControl::new(ctrl);
    let cache = refresh_cache(|state| ctrl.send(state)).await?;
    let ctrl = ctrl.into_inner();

    while let Some(msg) = ctrl.recv().await {
        match msg {
            ToServer::FsControl(fscontrol::BackToTheBeginning) => break,
            m => logger.invalid_message(&m),
        }
    }

    Ok(cache)
}

async fn filer_search_state(ctrl: &mut Control, cache: &Cache) -> MachineResult<()> {
    let logger = StateLogger::new("FilerSearch");

    ctrl.send(filer::search::search("".to_string(), cache))
        .await;

    while let Some(msg) = ctrl.recv().await {
        match msg {
            ToServer::FsControl(fscontrol::BackToTheBeginning) => break,
            ToServer::FsControl(fscontrol::Search(search)) => {
                ctrl.send(filer::search::search(search, cache)).await;
            }
            ToServer::MpvStart(mpvstart::File(file)) => {
                return Jump::mpv_file(file.root, file.path);
            }
            m => logger.invalid_message(&m),
        }
    }

    Ok(())
}
