#[macro_use]
mod util;
mod caster;
mod config;
mod connections;
mod filer;
mod job;
mod mpv;
mod process;
mod repeatable_oneshot;
mod signal;

use std::process::ExitCode;

use futures_util::future::maybe_done;
use protocol::Message;
use tokio::{join, select, spawn, sync::mpsc, task::JoinError};
use tokio_util::sync::CancellationToken;

use crate::{
    caster::caster_actor, connections::connections_actor, signal::signal_received,
};

const CHANNEL_SIZE: usize = 1024;
type Sender = mpsc::Sender<Message>;
type Receiver = mpsc::Receiver<Message>;

fn init_logger() {
    use simplelog::*;

    // TODO: detect if from systemd and inte skriva ut tider då
    // NOTE: set_time_offset_to_local can only be run when there is only on thread active.
    let config = match ConfigBuilder::new().set_time_offset_to_local() {
        Ok(builder) => builder,
        Err(builder) => {
            eprintln!(
                "Failed to set time zone for the logger, using UTC instead (I think)"
            );
            builder
        }
    }
    .add_filter_allow_str("server")
    .build();

    let level = LevelFilter::Debug;
    let colors = if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    TermLogger::init(level, config, TerminalMode::Stdout, colors)
        .expect("could not init logger");
}

fn log_actor_error(res: Result<Result<(), anyhow::Error>, JoinError>, name: &str) {
    match res {
        Err(je) => log::error!("Actor '{}' join error: {}", name, je),
        Ok(Err(ae)) => log::error!("Actor '{}' errored with: {:?}", name, ae),
        Ok(Ok(())) => (),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn async_main() -> ExitCode {
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
