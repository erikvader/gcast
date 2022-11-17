use std::io;

use protocol::{to_client::front, to_server::fscontrol::FsControl};

use crate::{
    job::handlejob::{HandleJob, HandleJobError},
    process::Process,
};

#[async_trait::async_trait]
impl HandleJob for Process {
    type State = front::Front;
    type Error = io::Error;
    type Ctrl = ();

    fn initial_state(&self) -> Self::State {
        front::Front::Spotify
    }

    fn name(&self) -> &str {
        Self::name(self)
    }

    async fn next(&mut self) -> Result<Self::State, Self::Error> {
        let res = self.wait().await;
        log::warn!("Process '{}' exited early with: {}", self.name(), res?);
        Ok(self.initial_state())
    }

    async fn wait_until_closed(mut self) {
        match self.wait().await {
            Ok(status) => {
                log::debug!("Process '{}' exited with: {}", self.name(), status)
            }
            Err(err) => {
                log::error!("Process '{}' exited with error: {}", self.name(), err)
            }
        }
    }

    async fn quit(&mut self) -> Result<(), Self::Error> {
        self.kill();
        Ok(())
    }

    async fn control(&mut self, _ctrl: Self::Ctrl) -> Result<(), Self::Error> {
        panic!("there is no way to send Ctrl(T) here");
    }
}

impl HandleJobError for io::Error {
    fn is_normal_exit(&self) -> bool {
        false
    }
}
