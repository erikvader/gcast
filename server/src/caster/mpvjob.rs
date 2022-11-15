use protocol::to_client::front;

use crate::{
    job::handlejob::{HandleJob, HandleJobError},
    mpv::{MpvError, MpvHandle},
};

#[async_trait::async_trait]
impl HandleJob for MpvHandle {
    type State = front::mpv::Mpv;
    type Error = MpvError;
    type Ctrl = protocol::to_server::mpvcontrol::MpvControl;

    fn initial_state(&self) -> Self::State {
        front::mpv::Load
    }

    fn name(&self) -> &str {
        "mpv"
    }

    async fn next(&mut self) -> Result<Self::State, Self::Error> {
        loop {
            match Self::next(self).await.map(|s| s.to_client_state()) {
                Ok(Some(s)) => break Ok(s),
                Ok(None) => (),
                Err(e) => break Err(e),
            }
        }
    }

    async fn wait_until_closed(self) {
        Self::wait_until_closed(self).await
    }

    async fn quit(&mut self) -> Result<(), Self::Error> {
        Self::quit(self).await
    }

    async fn control(&mut self, ctrl: Self::Ctrl) -> Result<(), Self::Error> {
        self.command(&ctrl).await
    }
}

impl HandleJobError for MpvError {
    fn is_normal_exit(&self) -> bool {
        matches!(self, MpvError::Exited)
    }
}
