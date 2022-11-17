use protocol::ToMessage;
use tokio::select;

use crate::{job::JobMsg, util::send_to_conn, Sender};

#[async_trait::async_trait]
pub trait HandleJob {
    type State: ToMessage + Send + Sync + Clone;
    type Error: HandleJobError + Send + 'static;
    type Ctrl: Send + 'static;

    fn initial_state(&self) -> Self::State;
    fn name(&self) -> &str;
    async fn next(&mut self) -> Result<Self::State, Self::Error>;
    async fn wait_until_closed(self);
    async fn quit(&mut self) -> Result<(), Self::Error>;
    async fn control(&mut self, ctrl: Self::Ctrl) -> Result<(), Self::Error>;
}

pub trait HandleJobError {
    fn is_normal_exit(&self) -> bool;
}

pub fn handle_job_start<T, F, E>(
    to_conn: Sender,
    create_handle: F,
) -> super::Job<T::Ctrl, T::Error>
where
    F: FnOnce() -> Result<T, E> + Send + 'static,
    E: Into<T::Error>,
    T: HandleJob + Send,
{
    super::Job::start(|mut rx| async move {
        let mut handle = create_handle().map_err(|e| e.into())?;

        let mut last_state = handle.initial_state();
        send_to_conn(&to_conn, last_state.clone()).await;

        let retval = loop {
            select! {
                msg = rx.recv() => {
                    match msg {
                        None => {
                            log::debug!("Exit signal for handle '{}' received", handle.name());
                            break handle.quit().await
                        },
                        Some(JobMsg::SendStatus) => send_to_conn(&to_conn, last_state.clone()).await,
                        Some(JobMsg::Ctrl(ctrl)) => break_err!(handle.control(ctrl).await),
                    }
                }
                state = handle.next() => {
                    match state {
                        Ok(newstate) => {
                            last_state = newstate.clone();
                            send_to_conn(&to_conn, newstate).await;
                        }
                        Err(e) if e.is_normal_exit() => break Ok(()),
                        Err(e) => break Err(e),
                    }
                }
            }
        };

        log::debug!("Waiting for handle '{}' to exit", handle.name());
        handle.wait_until_closed().await;
        retval
    })
}
