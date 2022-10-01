use clap::{Parser, Subcommand};
use protocol::{to_server::spotifystart, Message, ToMessage};
use tungstenite::connect;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    SpotifyStart,
    SpotifyStop,
}

fn init_logger() {
    use simplelog::*;

    let level = LevelFilter::Debug;
    let config = ConfigBuilder::new()
        .add_filter_allow_str("test_client")
        .build();
    let colors = ColorChoice::Auto;

    TermLogger::init(level, config, TerminalMode::Stdout, colors)
        .expect("could not init logger");
}

fn main() {
    init_logger();
    let cli = Cli::parse();

    let target_url = "ws://127.0.0.1:1337";
    log::info!("Connecting to {}...", target_url);

    let (mut socket, response) = connect(target_url).expect("connect failed");

    log::info!("Connected!");
    log::debug!("response: {:?}", response);

    let tosend = match &cli.command {
        Commands::SpotifyStart => spotifystart::Start.to_message(),
        Commands::SpotifyStop => spotifystart::Stop.to_message(),
    };
    log::info!("Sending: {:?}", tosend);
    let data = tosend.serialize().expect("serialization failed");
    socket
        .write_message(tungstenite::Message::Binary(data))
        .expect("could not send");
    log::info!("Sent");

    log::info!("Listening for all replies...");
    loop {
        let msg = socket.read_message().expect("could not read");
        log::info!("Received raw: {:?}", msg);

        if msg.is_close() {
            break;
        }

        if let tungstenite::Message::Binary(data) = msg {
            let msg = Message::deserialize(&data).expect("could not deserialize");
            log::info!("Received parsed: {:?}", msg);
        }
    }

    socket.close(None).expect("failed to close");
    // std::thread::sleep(std::time::Duration::from_secs(5));
    log::info!("bye");
}
