use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Context;
use futures_util::{Sink, SinkExt, StreamExt, TryStreamExt};
use protocol::{to_client::seat::Seat, Message, ToClientable};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};
use tokio_tungstenite::tungstenite::Message as TungMsg;
use tokio_util::sync::CancellationToken;

use crate::{
    config,
    util::{join_handle_wait_take, FutureCancel},
};

type Sender = mpsc::Sender<protocol::ToServer>;
type Receiver = mpsc::Receiver<protocol::ToClient>;

mod tractor;

async fn ws_send<T, S>(msg: T, ws: &mut S) -> anyhow::Result<()>
where
    T: ToClientable,
    S: Sink<TungMsg> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let bytes = protocol::Message::from(msg.to_client()).serialize()?;
    ws.send(TungMsg::Binary(bytes)).await?;
    Ok(())
}

async fn reject(tcp_stream: TcpStream) -> anyhow::Result<()> {
    let mut ws = tokio_tungstenite::accept_async(tcp_stream).await?;
    ws_send(Seat::Reject, &mut ws).await?;
    ws.close(None).await?;
    Ok(())
}

async fn handle_rejections(
    listener: TcpListener,
    canceltoken: CancellationToken,
) -> anyhow::Result<TcpListener> {
    log::info!("Rejecting new connections");
    loop {
        log::debug!("Waiting for new connection to reject...");
        let (stream, addr) = match listener.accept().cancellable(&canceltoken).await {
            Some(x) => x.context("failed to accept tcp stream")?,
            None => {
                log::debug!("Handle_rejections got cancelled");
                return Ok(listener);
            }
        };

        log::info!("Rejecting {}...", addr);
        if let Err(e) = reject(stream).await {
            log::warn!("Did not reject {} successfully cuz {:?}", addr, e);
        } else {
            log::info!("Rejected {}", addr);
        }
    }
}

async fn throw_away(mut from_cast: Receiver, canceltoken: CancellationToken) -> Receiver {
    log::debug!("Throwing away messages from caster");
    loop {
        match from_cast.recv().cancellable(&canceltoken).await {
            None => break from_cast,
            Some(Some(msg)) => log::trace!("Threw away msg: {:?}", msg),
            Some(None) => {
                log::warn!("Caster seems to be down");
                break from_cast;
            }
        };
    }
}

async fn handle_accept(
    tcp_stream: TcpStream,
    addr: SocketAddr,
    to_cast: &mut Sender,
    from_cast: &mut Receiver,
    canceltoken: CancellationToken,
) -> anyhow::Result<()> {
    log::info!("Accepting connection from: {}", addr);
    let ws = tokio_tungstenite::accept_async(tcp_stream).await?;
    let (mut sink, mut stream) = ws.split();

    log::debug!("Sending accept...");
    ws_send(Seat::Accept, &mut sink).await?;

    let tractor = tractor::Tractor::new(sink);

    loop {
        select! {
            next = stream.try_next() => {
                match next? {
                    None => {
                        log::info!("Client closed");
                        break;
                    },
                    Some(TungMsg::Binary(msg)) => handle_binary(&msg, to_cast).await,
                    Some(TungMsg::Close(msg)) => log::debug!("Closed message with '{:?}'", msg),
                    Some(msg) => log::warn!("Got a non-binary message {:?}", msg),
                }
            },
            _ = canceltoken.cancelled() => {
                log::debug!("Handle_accept got cancelled");
                break;
            },
            Some(msg) = from_cast.recv() => if tractor.send(msg).await.is_err() {
                break
            },
        }
    }

    let mut sink = tractor.close().await?;
    log::debug!("Disconnecting: {}", addr);
    if let Err(e) = sink.close().await {
        log::warn!(
            "Failed to send close message, client probably already disconnected: {}",
            e
        );
    }
    log::info!("Disconnected: {}", addr);
    Ok(())
}

async fn handle_binary(msg: &[u8], to_cast: &mut Sender) {
    match Message::deserialize(msg) {
        Err(e) => log::warn!("Failed to deserialize message {:?} cuz {}", &msg, e),
        Ok(m) if m.is_to_server() => {
            log::debug!("Received (caster): {:?}", m);
            if to_cast.send(m.take_to_server().unwrap()).await.is_err() {
                log::warn!("Seems like caster is down");
            }
        }
        Ok(m) => log::warn!("Received a message to client: {:?}", m),
    }
}

pub async fn connections_actor(
    mut to_cast: Sender,
    mut from_cast: Receiver,
    canceltoken: CancellationToken,
) -> anyhow::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config::port());
    let mut listener = TcpListener::bind(addr).await?;
    log::info!("Listening on: {}", addr);

    loop {
        let throw_token = canceltoken.child_token();
        let throw_handle = tokio::spawn(throw_away(from_cast, throw_token.clone()));

        log::debug!("Waiting for a new connection to accept...");
        let (stream, addr) = match listener.accept().cancellable(&canceltoken).await {
            Some(x) => x.context("failed to accept tcp stream")?,
            None => {
                log::debug!("Connections is aborting at accept with no one connected...");
                break;
            }
        };

        log::debug!("Cancelling and waiting for throw_handle to exit...");
        throw_token.cancel();
        from_cast = join_handle_wait_take(throw_handle).await;

        let rejections_token = canceltoken.child_token();
        let rej_handle =
            tokio::spawn(handle_rejections(listener, rejections_token.clone()));
        if let Err(e) = handle_accept(
            stream,
            addr,
            &mut to_cast,
            &mut from_cast,
            canceltoken.child_token(),
        )
        .await
        {
            log::warn!("Handle_accept exited with an error: {:?}", e);
        }

        log::debug!("Cancelling and waiting for handle_rejections to exit...");
        rejections_token.cancel();
        listener = join_handle_wait_take(rej_handle)
            .await
            .context("handle_rejections failed to give back TcpListener")?;
    }

    log::info!("Connections actor exited");
    Ok(())
}
