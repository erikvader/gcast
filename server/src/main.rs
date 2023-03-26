#[macro_use]
mod util;
mod caster;
mod config;
mod connections;
mod filer;
mod mpv;
mod process;
mod repeatable_oneshot;
mod signal;
mod state_machine;

use std::process::ExitCode;

use futures_util::future::maybe_done;
use protocol::Message;
use tokio::{join, select, spawn, sync::mpsc, task::JoinError};
use tokio_util::sync::CancellationToken;

use crate::{
    caster::caster_actor, connections::connections_actor, signal::signal_received,
};

const CHANNEL_SIZE: usize = 1024;
// TODO: rename to MsgSender and MsgReceiver?
type Sender = mpsc::Sender<Message>;
type Receiver = mpsc::Receiver<Message>;

fn started_by_systemd() -> bool {
    // TODO: use https://docs.rs/systemd-journal-logger/latest/systemd_journal_logger/index.html instead?
    // NOTE: man 5 systemd.exec
    match std::env::var("JOURNAL_STREAM") {
        Ok(_) | Err(std::env::VarError::NotUnicode(_)) => true,
        Err(std::env::VarError::NotPresent) => false,
    }
}

fn init_logger() {
    use simplelog::*;

    let mut builder = ConfigBuilder::new();
    builder.add_filter_allow_str("server");

    if started_by_systemd() {
        builder.set_time_level(LevelFilter::Off);
    } else {
        // NOTE: set_time_offset_to_local can only be run when there is only on thread active.
        if builder.set_time_offset_to_local().is_err() {
            eprintln!(
                "Failed to set time zone for the logger, using UTC instead (I think)"
            );
        }
    }

    let level = LevelFilter::Debug;
    let colors = if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    TermLogger::init(level, builder.build(), TerminalMode::Stdout, colors)
        .expect("could not init logger");
}

fn log_actor_error(res: Result<Result<(), anyhow::Error>, JoinError>, name: &str) {
    match res {
        Err(je) => log::error!("Actor '{}' join error: {}", name, je),
        Ok(Err(ae)) => log::error!("Actor '{}' errored with: {:?}", name, ae),
        Ok(Ok(())) => (),
    }
}

async fn maybe_refresh_cache() {
    if !config::refresh_cache_boot() {
        log::debug!("Not refreshing cache on start since it is not configured");
        return;
    }

    if std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(std::path::PathBuf::from(format!(
            "/tmp/{}_cache_initialized",
            config::PROGNAME
        )))
        .is_ok()
    {
        log::info!("Refreshing the cache at startup");
        if let Err(e) = filer::refresh_cache_at_init().await {
            log::error!("Failed to refresh the cache at initalization time: {}", e);
        }
    } else {
        log::info!(
            "Not refreshing the cache at startup, it has already been done this boot"
        );
    }
}

#[tokio::main(flavor = "current_thread")]
async fn async_main() -> ExitCode {
    maybe_refresh_cache().await;

    let (to_cast, from_conn) = mpsc::channel(CHANNEL_SIZE);
    let (to_conn, from_cast) = mpsc::channel(CHANNEL_SIZE);
    let canceltoken = CancellationToken::new();

    let caster = maybe_done(spawn(caster_actor(
        to_cast,
        from_cast,
        canceltoken.child_token(),
    )));
    let connections = maybe_done(spawn(connections_actor(
        to_conn,
        from_conn,
        canceltoken.child_token(),
    )));
    tokio::pin!(caster);
    tokio::pin!(connections);

    select! {
        _ = signal_received() => log::info!("Terminating due to a signal"),
        _ = &mut caster => log::error!("Caster actor terminated early"),
        _ = &mut connections => log::error!("Connections actor terminated early"),
    }

    log::info!("Cancelling remaining actors...");
    canceltoken.cancel();

    join!(&mut caster, &mut connections);
    match (
        caster.take_output().expect("value not taken"),
        connections.take_output().expect("value not taken"),
    ) {
        (Ok(Ok(())), Ok(Ok(()))) => log::info!("Tasks exited normally"),
        (r1, r2) => {
            log::error!("Some actor exited abnormally");
            log_actor_error(r1, "caster");
            log_actor_error(r2, "connections");
        }
    }

    log::info!("Goodbye");
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    init_logger();
    log::info!("Welcome");

    if let Err(e) = config::init_config() {
        log::error!("Failed to read config: {:?}", e);
        return ExitCode::FAILURE;
    }

    async_main()
}
