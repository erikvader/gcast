use anyhow::Context;
use protocol::{
    to_client::front::filesearch,
    to_client::front::filesearch::tree as prot_tree,
    to_server::{
        fscontrol::{self, search_ctrl, tree_ctrl},
        fsstart, mpvstart, ToServer,
    },
};

use crate::filer::{
    self, cache::Cache, cache_file, read_cache, refresh_cache, tree::Tree,
};

use super::{Control, Jump, LockedControl, MachineResult, StateLogger};

pub(super) async fn filer_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("Filer");
    let mut cache = filer_read_cache_state(ctrl).await?;

    while let Some(msg) = ctrl
        .send_recv_lazy(|| filesearch::init::Init {
            last_cache_date: cache.updated(),
        })
        .await
    {
        match msg {
            ToServer::FsStart(fsstart::Stop) => break,
            ToServer::FsStart(fsstart::RefreshCache) => {
                cache = filer_refresh_cache_state(ctrl)
                    .await
                    .context("filer refresh cache state")?;
            }
            ToServer::FsStart(fsstart::Search) => {
                filer_search_state(ctrl, &cache)
                    .await
                    .context("filer search state")?;
            }
            ToServer::FsStart(fsstart::Tree) => {
                filer_tree_state(ctrl, &cache)
                    .await
                    .context("filer tree state")?;
            }
            m => logger.invalid_message(&m),
        }
    }

    Ok(())
}

async fn filer_read_cache_state(ctrl: &mut Control) -> MachineResult<Cache> {
    let _logger = StateLogger::new("FilerReadCache");

    // TODO: flag to say it is initializing/loading
    ctrl.send(filesearch::init::Init {
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
            ToServer::FsStart(fsstart::Stop) => break,
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
            ToServer::FsStart(fsstart::Stop) => break,
            ToServer::FsControl(fscontrol::SearchCtrl(search_ctrl::Search(search))) => {
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

async fn filer_tree_state(ctrl: &mut Control, cache: &Cache) -> MachineResult<()> {
    let logger = StateLogger::new("FilerTree");
    let mut tree = Tree::new(cache);

    while let Some(msg) = ctrl.send_recv(create_tree_state(&tree)).await {
        match msg {
            ToServer::FsStart(fsstart::Stop) => break,
            ToServer::FsControl(fscontrol::TreeCtrl(tree_ctrl::Cd(i))) => {
                if let Err(()) = tree.cd(i) {
                    logger.error(format!("can't cd, invalid i={i}"));
                }
            }
            ToServer::FsControl(fscontrol::TreeCtrl(tree_ctrl::CdDotDot)) => {
                if let Err(()) = tree.cd_up() {
                    logger.warn("can't cd up, already at the top");
                }
            }
            ToServer::MpvStart(mpvstart::File(file)) => {
                return Jump::mpv_file(file.root, file.path);
            }
            m => logger.invalid_message(&m),
        }
    }

    Ok(())
}

fn create_tree_state(tree: &Tree) -> prot_tree::Tree {
    prot_tree::Tree {
        breadcrumbs: tree.breadcrumbs(),
        contents: tree
            .files()
            .map(|file| match file {
                filer::tree::File {
                    name,
                    path_relative_root,
                    ty: filer::tree::Type::Regular,
                    ..
                } => prot_tree::Entry::File {
                    path: path_relative_root.to_string(),
                    root: tree
                        .root()
                        .expect("this is non-None if there are files available"),
                    name: name.to_string(),
                },
                filer::tree::File {
                    name,
                    ty: filer::tree::Type::Directory,
                    id,
                    ..
                }
                | filer::tree::File {
                    name,
                    ty: filer::tree::Type::Root,
                    id,
                    ..
                } => prot_tree::Entry::Dir {
                    name: name.to_string(),
                    id,
                },
            })
            .collect(),
    }
}
