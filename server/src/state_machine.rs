use std::future::Future;
use std::{collections::VecDeque, convert::Infallible};

use anyhow::Context;
use protocol::to_server::playurlstart;
use protocol::{
    to_client::{front::Front, ToClient},
    to_server::{
        mpvstart::{self, MpvStart},
        ToServer,
    },
    Message, ToMessage,
};
use tokio_util::sync::CancellationToken;

use crate::util::FutureCancel;
use crate::{Receiver, Sender};

use self::mpv_play::{mpv_file_state, mpv_url_state};
use self::play_url::play_url_state;

mod error_msg;
mod mpv_play;
mod play_url;

pub type MachineResult<T> = anyhow::Result<T>;

struct Gatekeeper {
    last_sent: Front,
    next_id: protocol::Id,
}

impl Gatekeeper {
    fn new(initial_state: Front) -> Self {
        Self {
            last_sent: initial_state,
            next_id: 0,
        }
    }

    fn should_accept(&mut self, msg: &Message) -> bool {
        if msg.is_expected_or_newer_than(self.next_id) {
            self.next_id = msg.id() + 1;
            true
        } else {
            false
        }
    }

    fn last_sent(&self) -> Front {
        self.last_sent.clone()
    }

    fn set_last_sent(&mut self, msg: &Message) {
        assert!(msg.is_to_client(), "wrong message kind");
        if let ToClient::Front(f) = msg.borrow_to_client() {
            self.last_sent = f.clone();
        }
    }
}

struct Control {
    from_conn: Receiver,
    to_conn: Sender,
    keeper: Gatekeeper,
    canceltoken: CancellationToken,
}

impl Control {
    fn new(
        from_conn: Receiver,
        to_conn: Sender,
        initial_state: Front,
        canceltoken: CancellationToken,
    ) -> Self {
        Self {
            from_conn,
            to_conn,
            keeper: Gatekeeper::new(initial_state),
            canceltoken,
        }
    }

    async fn send(&mut self, msg: impl ToMessage) {
        let m = msg.to_message();
        self.keeper.set_last_sent(&m);
        if self.to_conn.send(m).await.is_err() {
            log::warn!("Seems like connections is down");
        }
    }

    async fn recv(&mut self) -> Option<ToServer> {
        while let Some(Some(msg)) =
            self.from_conn.recv().cancellable(&self.canceltoken).await
        {
            assert!(msg.is_to_server(), "connections actor's responsibility");
            if !self.keeper.should_accept(&msg) {
                log::debug!("Throwing away an out of date message");
                continue;
            }

            let toserver = msg.take_to_server();
            if let ToServer::SendStatus(_) = toserver {
                self.send(self.keeper.last_sent()).await;
                continue;
            }

            return Some(toserver);
        }

        log::info!("Connections closed its end or I got cancelled, exiting...");
        None
    }

    async fn send_recv(&mut self, msg: impl ToMessage) -> Option<ToServer> {
        self.send(msg).await;
        self.recv().await
    }
}

struct InjectableQueue<T> {
    queue: VecDeque<T>,
}

impl<T> InjectableQueue<T> {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn inject(&mut self, elt: T) {
        self.queue.push_back(elt);
    }

    async fn pop_or<Fut, F>(&mut self, fut: F) -> Option<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<T>>,
    {
        match self.queue.pop_front() {
            Some(x) => Some(x),
            None => fut().await,
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum Jump {
    #[error("Jump to mpv")]
    Mpv(MpvStart),
    #[error("Jump to user error")]
    UserError { header: String, body: String },
}

impl Jump {
    fn user_error<H, E>(header: H, error: E) -> MachineResult<()>
    where
        H: ToString,
        E: std::fmt::Debug,
    {
        Err(Self::UserError {
            header: header.to_string(),
            body: format!("{:?}", error),
        }
        .into())
    }

    fn mpv_file(root: usize, path: String) -> MachineResult<()> {
        Err(Self::Mpv(mpvstart::File { root, path }.into()).into())
    }

    fn mpv_url(url: String) -> MachineResult<()> {
        Err(Self::Mpv(mpvstart::Url(url).into()).into())
    }
}

struct StateLogger<'a> {
    name: &'a str,
}

impl<'a> StateLogger<'a> {
    fn new(name: &'a str) -> Self {
        log::info!("Entered state '{}'", name);
        Self { name }
    }

    fn invalid_message(&self, msg: &ToServer) {
        log::warn!("State '{}' received an invalid msg: {:?}", self.name, msg);
    }

    fn attempt_exit(&self) {
        log::debug!("State '{}' is attempting to exit", self.name);
    }

    fn waiting(&self, something: &str) {
        log::debug!("State '{}' is waiting for {}", self.name, something);
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl Drop for StateLogger<'_> {
    fn drop(&mut self) {
        log::info!("Exited state '{}'", self.name);
    }
}

pub async fn state_start(
    from_conn: Receiver,
    to_conn: Sender,
    canceltoken: CancellationToken,
) -> MachineResult<()> {
    let mut ctrl = Control::new(from_conn, to_conn, Front::None, canceltoken);
    init_state(&mut ctrl).await
}

async fn init_state(ctrl: &mut Control) -> MachineResult<()> {
    let logger = StateLogger::new("Init");
    let mut queue = InjectableQueue::new();

    while let Some(msg) = queue.pop_or(|| ctrl.send_recv(Front::None)).await {
        let res: MachineResult<()> = match msg {
            ToServer::PowerCtrl(_) => todo!(),
            ToServer::MpvStart(mpvstart::Url(url)) => {
                mpv_url_state(ctrl, url).await.context("mpv url")
            }
            ToServer::MpvStart(mpvstart::File(file)) => {
                mpv_file_state(ctrl, file.root, file.path)
                    .await
                    .context("mpv file")
            }
            ToServer::SpotifyStart(_) => todo!(),
            ToServer::FsStart(_) => todo!(),
            ToServer::PlayUrlStart(playurlstart::Start) => {
                play_url_state(ctrl).await.context("play url")
            }
            _ => {
                logger.invalid_message(&msg);
                Ok(())
            }
        };

        if let Err(e) = res.context(format!("in state '{}'", logger.name())) {
            match e.downcast() {
                Ok(Jump::Mpv(mpvstart)) => queue.inject(mpvstart.into()),
                Ok(Jump::UserError { header, body }) => {
                    error_msg::error_msg_state(ctrl, header, body)
                        .await
                        .context("error message")?
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok(())
}
