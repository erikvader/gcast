#[macro_use]
mod util;
mod caster;
mod config;
mod connections;
mod filer;
mod mpv;
mod process;
mod signal;
mod state_machine;

use std::process::ExitCode;

use futures_util::future::maybe_done;
use tokio::{join, select, spawn, sync::mpsc, task::JoinError};
use tokio_util::sync::CancellationToken;

use crate::{
    caster::caster_actor, connections::connections_actor, signal::signal_received,
};

const CHANNEL_SIZE: usize = 1024;

fn init_logger() {
    use log::LevelFilter;
    use systemd_journal_logger::{connected_to_journal, JournalLog};

    const SERVER: &str = "server";

    fn install_systemd_logger() -> bool {
        struct FilteringJournalLog(JournalLog);
        impl log::Log for FilteringJournalLog {
            fn enabled(&self, metadata: &log::Metadata) -> bool {
                log::Log::enabled(&self.0, metadata)
            }

            fn log(&self, record: &log::Record) {
                if self.enabled(record.metadata()) {
                    if record.target().starts_with(SERVER) {
                        log::Log::log(&self.0, record);
                    }
                }
            }

            fn flush(&self) {
                log::Log::flush(&self.0)
            }
        }

        let logger = match JournalLog::new() {
            Ok(logger) => logger,
            Err(e) => {
                eprintln!("Failed to create the systemd logger: {e:?}");
                return false;
            }
        };

        log::set_max_level(LevelFilter::Trace);
        log::set_boxed_logger(Box::new(FilteringJournalLog(logger)))
            .expect("no logger should have been set yet");
        true
    }

    fn install_stdout_logger() {
        use fern_format::{Format, Stream};
        fern::Dispatch::new()
            .level(LevelFilter::Off)
            .level_for(SERVER, LevelFilter::Trace)
            .format(Format::new().color_if_supported(Stream::Stdout).callback())
            .chain(std::io::stdout())
            .apply()
            .expect("no logger should have been set yet");
    }

    if !connected_to_journal() || !install_systemd_logger() {
        install_stdout_logger();
    }
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

// TODO: Beware of misbehaving tasks that block for too long. Disabling the LIFO slot did
// help, but is not a solution. This is just a PSA. This comment can be removed when that
// special slot is stealable. The mpv issue is maybe caused by this?
// https://github.com/tokio-rs/tokio/issues/4941
// https://github.com/tokio-rs/tokio/issues/6315#issuecomment-1920876711
#[tokio::main]
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
