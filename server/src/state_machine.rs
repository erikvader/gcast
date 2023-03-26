use std::collections::VecDeque;
use std::future::Future;
use std::process::ExitStatus;

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

mod init_state;

pub type MachineResult<T> = anyhow::Result<T>;

struct Gatekeeper {
    last_sent: Front,
}

impl Gatekeeper {
    fn new(initial_state: Front) -> Self {
        Self {
            last_sent: initial_state,
        }
    }

    fn should_accept(&mut self, _msg: &Message) -> bool {
        // TODO: this doesn't work! Needs to be reset on each new client connection. This
        // logic thus needs to be in connections.rs.

        // if msg.is_expected_or_newer_than(self.next_id) {
        //     self.next_id = msg.id() + 1;
        //     true
        // } else {
        //     false
        // }
        true
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
                log::debug!("Sending the last sent state again");
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

    async fn send_recv_lazy<F, M>(&mut self, msg_fun: F) -> Option<ToServer>
    where
        F: FnOnce() -> M,
        M: ToMessage,
    {
        self.send(msg_fun()).await;
        self.recv().await
    }
}

// It is problematic to create an async closure that captures stuff mutably.
//
// "If I were able to define an async closure mutably capturing its environment, it
// would be possible to invoke the closure multiple times without actually awaiting
// the future (or dropping it in some other way).
// This way, we would get multiple Futures with aliased mutable pointers."
// Source: https://github.com/rust-lang/rust/issues/69446#issuecomment-619354375
//
// This is a wrapper around Control that makes it possible by providing a non-mutable send
// function. The send function is guarded by a mutex instead of borrowing rules.
struct LockedControl<'a> {
    ctrl: tokio::sync::Mutex<&'a mut Control>,
}

impl<'a> LockedControl<'a> {
    fn new(ctrl: &'a mut Control) -> Self {
        Self {
            ctrl: tokio::sync::Mutex::new(ctrl),
        }
    }

    async fn send(&self, msg: impl ToMessage) {
        self.ctrl.lock().await.send(msg).await
    }

    fn into_inner(self) -> &'a mut Control {
        self.ctrl.into_inner()
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
    fn user_error<T, H, E>(header: H, error: E) -> MachineResult<T>
    where
        H: ToString,
        E: std::fmt::Debug,
    {
        let header = header.to_string();
        log::debug!("Jump to user error: header={}", header);
        Err(Self::UserError {
            header,
            body: format!("{:?}", error),
        }
        .into())
    }

    fn mpv_file<T>(root: usize, path: String) -> MachineResult<T> {
        log::debug!("Jump to mpv: root={}, path={}", root, path);
        Err(Self::Mpv(mpvstart::File { root, path }.into()).into())
    }

    fn mpv_url<T>(url: String) -> MachineResult<T> {
        log::debug!("Jump to mpv: url={}", url);
        Err(Self::Mpv(mpvstart::Url(url).into()).into())
    }
}

trait JumpableError<S, H> {
    fn jump_user_error(self, header: H) -> MachineResult<S>;
}

impl<T, H, E> JumpableError<T, H> for Result<T, E>
where
    E: std::fmt::Debug,
    H: ToString,
{
    fn jump_user_error(self, header: H) -> MachineResult<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Jump::user_error(header, e),
        }
    }
}

struct StateLogger<'a> {
    name: &'a str,
}

impl<'a> StateLogger<'a> {
    fn new(name: &'a str) -> Self {
        let seld = Self { name };
        seld.debug(format!("entered state"));
        seld
    }

    fn invalid_message(&self, msg: &ToServer) {
        self.warn(format!("received an invalid msg: {:?}", msg));
    }

    fn attempt_exit(&self) {
        self.debug("attempting to exit");
    }

    fn waiting(&self, something: impl AsRef<str>) {
        self.debug(format!("waiting for {}", something.as_ref()));
    }

    fn process_done(&self, proc_name: impl AsRef<str>, exit: ExitStatus) {
        if exit.success() {
            self.info(format!(
                "process '{}' exited with: {}",
                proc_name.as_ref(),
                exit
            ));
        } else {
            self.warn(format!(
                "process '{}' exited with: {}",
                proc_name.as_ref(),
                exit
            ));
        }
    }

    fn error(&self, something: impl AsRef<str>) {
        log::error!("State '{}': {}", self.name, something.as_ref());
    }

    fn info(&self, something: impl AsRef<str>) {
        log::info!("State '{}': {}", self.name, something.as_ref());
    }

    fn warn(&self, something: impl AsRef<str>) {
        log::warn!("State '{}': {}", self.name, something.as_ref());
    }

    fn debug(&self, something: impl AsRef<str>) {
        log::debug!("State '{}': {}", self.name, something.as_ref());
    }

    fn name(&self) -> &str {
        self.name
    }
}

impl Drop for StateLogger<'_> {
    fn drop(&mut self) {
        self.debug("exited state")
    }
}

pub async fn state_start(
    from_conn: Receiver,
    to_conn: Sender,
    canceltoken: CancellationToken,
) -> MachineResult<()> {
    let mut ctrl = Control::new(from_conn, to_conn, Front::None, canceltoken);
    init_state::init_state(&mut ctrl).await
}
