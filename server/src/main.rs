mod caster;
mod connections;
mod signal;

use futures_util::future::maybe_done;
use protocol::Message;
use tokio::{join, select, spawn, sync::mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    caster::caster_actor, connections::connections_actor, signal::signal_received,
};

// TODO: en klient åt gången
// TODO: ws_recv

const CHANNEL_SIZE: usize = 1024;
type Sender = mpsc::Sender<Message>;
type Receiver = mpsc::Receiver<Message>;

fn init_logger() {
    use simplelog::*;

    let level = LevelFilter::Debug;
    let config = ConfigBuilder::new().add_filter_allow_str("server").build();
    let colors = if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    TermLogger::init(level, config, TerminalMode::Stdout, colors)
        .expect("could not init logger");
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    log::info!("Welcome");
    init_logger();

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
        caster.take_output().expect("value not taken yet"),
        connections.take_output().expect("value not taken yet"),
    ) {
        (Ok(_), Ok(_)) => log::info!("Tasks exited normally"),
        (r1, r2) => log::error!(
            "Something exited with error: caster={:?}, connections={:?}",
            r1,
            r2
        ),
    }

    log::info!("Goodbye");
}
