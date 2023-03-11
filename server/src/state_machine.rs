use std::future::Future;
use std::{collections::VecDeque, convert::Infallible};

use anyhow::Context;
use protocol::{
    to_client::{front::Front, ToClient},
    to_server::{
        mpvstart::{self, MpvStart},
        ToServer,
    },
    Message, ToMessage,
};

use crate::{Receiver, Sender};

mod error_msg;

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
        assert!(!msg.is_to_client(), "wrong message kind");
        if let ToClient::Front(f) = msg.borrow_to_client() {
            self.last_sent = f.clone();
        }
    }
}

struct Control {
    from_conn: Receiver,
    to_conn: Sender,
    keeper: Gatekeeper,
}

impl Control {
    fn new(from_conn: Receiver, to_conn: Sender, initial_state: Front) -> Self {
        Self {
            from_conn,
            to_conn,
            keeper: Gatekeeper::new(initial_state),
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
        while let Some(msg) = self.from_conn.recv().await {
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

        log::info!("Connections closed its end, exiting...");
        None
    }

    async fn send_recv(&mut self, msg: impl ToMessage) -> Option<ToServer> {
        self.send(msg).await;
        self.recv().await
    }

    // TODO: göra såhär 1?
    fn jump_mpv_file(&self, root: usize, path: String) -> MachineResult<Infallible> {
        return Err(Jump::Mpv(mpvstart::File { root, path }.into()).into());
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
    // TODO: eller så här 2?
    fn user_error<H, E>(header: H, error: E) -> Self
    where
        H: ToString,
        E: std::fmt::Debug,
    {
        Self::UserError {
            header: header.to_string(),
            body: format!("{:?}", error),
        }
    }
}

fn log_state_exited(state_name: &str) {
    log::info!("State '{}' exited", state_name);
}

fn log_state_entered(state_name: &str) {
    log::info!("Entered state '{}'", state_name);
}

fn log_invalid_msg(state_name: &str, msg: &ToServer) {
    log::warn!("State '{}' received an invalid msg: {:?}", state_name, msg);
}

pub async fn state_start(from_conn: Receiver, to_conn: Sender) -> MachineResult<()> {
    let mut ctrl = Control::new(from_conn, to_conn, Front::None);
    init_state(&mut ctrl).await
}

async fn init_state(ctrl: &mut Control) -> MachineResult<()> {
    const NAME: &str = "Init";
    log_state_entered(NAME);

    let mut queue = InjectableQueue::new();

    while let Some(msg) = queue.pop_or(|| ctrl.send_recv(Front::None)).await {
        let res: MachineResult<()> = match msg {
            ToServer::PowerCtrl(_) => todo!(),
            ToServer::MpvStart(_) => todo!(),
            ToServer::SpotifyStart(_) => todo!(),
            ToServer::FsStart(_) => todo!(),
            ToServer::PlayUrlStart(_) => todo!(),
            _ => {
                log_invalid_msg(NAME, &msg);
                Ok(())
            }
        };

        if let Err(e) = res.context(format!("in state '{}'", NAME)) {
            match e.downcast() {
                Ok(Jump::Mpv(mpvstart)) => queue.inject(mpvstart.into()),
                Ok(Jump::UserError { header, body }) => {
                    error_msg::error_msg_state(ctrl, header, body)
                        .await
                        .context(format!(
                            "in state '{}' showing an error message",
                            NAME
                        ))?
                }
                Err(e) => return Err(e),
            }
        }
    }

    log_state_exited(NAME);
    Ok(())
}
