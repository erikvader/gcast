use protocol::{Message, ToMessage};
use tungstenite::connect;

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

    let target_url = "ws://127.0.0.1:1337";
    log::info!("Connecting to {}...", target_url);

    let (mut socket, response) = connect(target_url).expect("connect failed");

    log::info!("Connected!");
    log::debug!("response: {:?}", response);

    let tosend = protocol::to_server::sendstatus::SendStatus.to_message();
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

    log::info!("bye");
}
