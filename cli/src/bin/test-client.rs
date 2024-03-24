use clap::{Parser, Subcommand};
use protocol::{
    to_client::{seat, ToClient},
    to_server::{
        fscontrol::{self, search_ctrl, tree_ctrl},
        fsstart, mpvcontrol, mpvstart, sendstatus, spotifystart,
    },
    ToServerable,
};
use tungstenite::connect;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long)]
    listen: bool,
}

#[derive(Subcommand, Clone)]
enum Commands {
    SpotifyStart,
    SpotifyStop,
    FilerStart,
    FilerStop,
    FilerRefreshCache,
    FilerSearch { query: String },
    FilerTree,
    FilerCd { i: usize },
    FilerCdUp,
    MpvPlayUrl { url: String },
    MpvPlayFile { root: usize, path: String },
    MpvStop,
    MpvPause,
}

fn init_logger() {
    use fern_format::{Format, Stream};
    fern::Dispatch::new()
        .level(log::LevelFilter::Off)
        .level_for("test_client", log::LevelFilter::Trace)
        .format(Format::new().color_if_supported(Stream::Stdout).callback())
        .chain(std::io::stdout())
        .apply()
        .expect("no logger should have been set yet");
}

fn main() {
    init_logger();
    let cli = Cli::parse();

    let target_url = "ws://127.0.0.1:1337";
    log::info!("Connecting to {}...", target_url);

    let (mut socket, response) = connect(target_url).expect("connect failed");

    log::info!("Connected!");
    log::debug!("Response: {:?}", response);

    if !read_seat(&mut socket) {
        return;
    }

    send_read_state(&mut socket);

    let tosend: protocol::Message = match &cli.command {
        Commands::SpotifyStart => spotifystart::Start.to_server(),
        Commands::SpotifyStop => spotifystart::Stop.to_server(),
        Commands::FilerStart => fsstart::Start.to_server(),
        Commands::FilerStop => fsstart::Stop.to_server(),
        Commands::FilerRefreshCache => fsstart::RefreshCache.to_server(),
        Commands::FilerSearch { query } => {
            search_ctrl::Search(query.to_string()).to_server()
        }
        Commands::MpvPlayUrl { url } => mpvstart::url::Url(url.clone()).to_server(),
        Commands::MpvPlayFile { root, path } => mpvstart::file::File {
            root: *root,
            path: path.to_string(),
        }
        .to_server(),
        Commands::MpvStop => mpvstart::Stop.to_server(),
        Commands::MpvPause => mpvcontrol::TogglePause.to_server(),
        Commands::FilerCd { i } => tree_ctrl::Cd(*i).to_server(),
        Commands::FilerCdUp => tree_ctrl::CdDotDot.to_server(),
        Commands::FilerTree => fsstart::Tree.to_server(),
    }
    .into();

    log::info!("Sending: {:?}", tosend);
    let data = tosend.serialize().expect("serialization failed");
    for _ in 0..1 {
        socket
            .write_message(tungstenite::Message::Binary(data.clone()))
            .expect("could not send");
        log::info!("Sent");
    }

    if cli.listen {
        log::info!("Listening for all replies...");
        loop {
            let msg = socket.read_message().expect("could not read");
            // log::info!("Received raw: {:?}", msg);

            if msg.is_close() {
                break;
            }

            log::info!("Received: {:#?}", parse_tung_msg(msg));
        }
    }

    socket.close(None).expect("failed to close");
    // std::thread::sleep(std::time::Duration::from_secs(5));
    log::info!("Bye");
}

fn parse_tung_msg(msg: tungstenite::Message) -> protocol::Message {
    if let tungstenite::Message::Binary(data) = msg {
        protocol::Message::deserialize(&data).expect("could not deserialize")
    } else {
        panic!("nani");
    }
}

type WS =
    tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>;

fn read_seat(socket: &mut WS) -> bool {
    loop {
        let msg = parse_tung_msg(socket.read_message().expect("could not read message"));
        match msg.take_to_client() {
            Ok(ToClient::Seat(seat::Accept)) => {
                log::info!("Got accepted");
                break true;
            }
            Ok(ToClient::Seat(seat::Reject)) => {
                log::warn!("Got rejected");
                break false;
            }
            _ => (),
        }
    }
}

fn send_read_state(socket: &mut WS) {
    let data = protocol::Message::from(sendstatus::SendStatus.to_server())
        .serialize()
        .expect("ser failed");
    socket
        .write_message(tungstenite::Message::Binary(data))
        .expect("could not send");

    loop {
        let msg = parse_tung_msg(socket.read_message().expect("could not read message"));
        if let Ok(ToClient::Front(f)) = msg.take_to_client() {
            log::info!("Got state {:?}", f);
            break;
        }
    }
}
