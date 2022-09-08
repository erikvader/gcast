use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures_util::{sink::SinkExt, StreamExt, TryStreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    signal::unix::{signal, SignalKind},
};
use tokio_tungstenite::tungstenite::Message;

const PORT: u16 = 1337;

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

async fn handle_connection(tcp_stream: TcpStream, addr: SocketAddr) {
    log::info!("Someone connected from: {}", addr);
    let ws = match tokio_tungstenite::accept_async(tcp_stream).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Could not create websocket stream: {}", e);
            return;
        }
    };
    log::info!("Websocket ready");

    let (mut sink, mut stream) = ws.split();
    sink.send(Message::text("hejsan"))
        .await
        .expect("failed to send");

    let recv = stream.try_next().await.expect("failed to receive");
    log::info!("I received: {:?}", recv);

    log::info!("Disconnecting: {}", addr);
}

async fn signal_received() {
    let mut sigint = signal(SignalKind::interrupt()).expect("sigint signal failed");
    let mut sigterm = signal(SignalKind::terminate()).expect("sigterm signal failed");
    select! {
        _ = sigint.recv() => {
            log::warn!("received sigint");
        },
        _ = sigterm.recv() => {
            log::warn!("received sigterm");
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_logger();

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), PORT);
    let listener = TcpListener::bind(addr).await.expect("failed to bind");
    log::info!("Listening on: {}", addr);

    loop {
        select! {
            _ = signal_received() => break,
            r = listener.accept() => {
                match r {
                    Ok((stream, addr)) => {
                        tokio::spawn(handle_connection(stream, addr));
                    },
                    Err(e) => {
                        log::error!("TCP accept failed: {}", e);
                        break;
                    }
                }
            }
        }
    }

    log::info!("Goodbye");
}
