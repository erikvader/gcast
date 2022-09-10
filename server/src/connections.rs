use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use futures_util::{Sink, SinkExt, StreamExt, TryStreamExt};
use protocol::Message;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message as TungMsg;
use tokio_util::sync::CancellationToken;

use crate::{Receiver, Sender};

const PORT: u16 = 1337;

#[derive(thiserror::Error, Debug)]
enum WsError<E> {
    #[error("serde error: {0}")]
    Msg(#[from] protocol::MessageError),
    #[error("websocket error: {0}")]
    Ws(E),
}

// TODO: move
async fn ws_send<T, S>(msg: T, ws: &mut S) -> Result<(), WsError<S::Error>>
where
    T: Into<Message>,
    S: Sink<TungMsg> + Unpin,
{
    let bytes = msg.into().serialize()?;
    ws.send(TungMsg::Binary(bytes))
        .await
        .map_err(|e| WsError::Ws(e))?;
    Ok(())
}

async fn handle_accept(
    tcp_stream: TcpStream,
    addr: SocketAddr,
    to_cast: Sender,
    from_cast: Receiver,
) -> (Sender, Receiver) {
    log::info!("Someone connected from: {}", addr);
    let ws = match tokio_tungstenite::accept_async(tcp_stream).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Could not create websocket stream: {}", e);
            return (to_cast, from_cast);
        }
    };
    log::info!("Websocket ready");

    let (mut sink, mut stream) = ws.split();
    ws_send(protocol::to_client::pong::Pong, &mut sink)
        .await
        .expect("failed to send");

    let recv = stream.try_next().await.expect("failed to receive");
    log::info!("I received: {:?}", recv);

    log::info!("Disconnecting: {}", addr);
    sink.close().await.expect("failed to close");
    log::info!("Disconnected: {}", addr);

    (to_cast, from_cast)
}

pub async fn connections_actor(
    mut to_cast: Sender,
    mut from_cast: Receiver,
    canceltoken: CancellationToken,
) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), PORT);
    let listener = TcpListener::bind(addr).await.expect("failed to bind");
    log::info!("Listening on: {}", addr);

    loop {
        let (stream, addr) = listener.accept().await.expect("TCP accept failed");
        let handle = tokio::spawn(handle_accept(stream, addr, to_cast, from_cast));
        let x = handle.await.unwrap();
        to_cast = x.0;
        from_cast = x.1;
    }
}
