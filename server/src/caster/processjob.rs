use std::{io, process::ExitStatus};

use protocol::to_client::front;

use crate::{
    job::handlejob::{HandleJob, HandleJobError},
    process::{Process, ProcessError},
};

#[derive(thiserror::Error, Debug)]
pub enum ProcessHandleJobError {
    #[error("Process error: {0}")]
    Proc(#[from] ProcessError),
    #[error("Process exited early with: {0}")]
    EarlyExit(ExitStatus),
}

#[async_trait::async_trait]
impl HandleJob for Process {
    type State = front::Front;
    type Error = ProcessHandleJobError;
    type Ctrl = ();

    fn initial_state(&self) -> Self::State {
        front::Front::Spotify
    }

    fn name(&self) -> &str {
        Self::name(self)
    }

    async fn next(&mut self) -> Result<Self::State, Self::Error> {
        let res = self.wait().await.expect("not allowed to be called twice")?;
        Err(ProcessHandleJobError::EarlyExit(res))
    }

    async fn wait_until_closed(mut self) {
        match self.wait().await {
            Some(Ok(status)) if status.success() => {
                log::info!("Process '{}' exited with: {}", self.name(), status)
            }
            Some(Ok(status)) => {
                log::warn!("Process '{}' exited with error: {}", self.name(), status)
            }
            Some(Err(err)) => {
                log::error!(
                    "Error while waiting for process '{}' to exit: {}",
                    self.name(),
                    err
                )
            }
            None => log::warn!("Process '{}' exited on its own", self.name()),
        }
    }

    async fn quit(&mut self) -> Result<(), Self::Error> {
        self.kill();
        Ok(())
    }

    async fn control(&mut self, _ctrl: Self::Ctrl) -> Result<(), Self::Error> {
        panic!("the received message must be a SendStatus");
    }
}

impl HandleJobError for ProcessHandleJobError {
    fn is_normal_exit(&self) -> bool {
        false
    }
}
