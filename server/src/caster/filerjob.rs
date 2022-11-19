use protocol::{to_client::front, to_server::fscontrol::FsControl};

use crate::{
    filer::{FilerError, Handle},
    job::handlejob::{HandleJob, HandleJobError},
};

#[async_trait::async_trait]
impl HandleJob for Handle {
    type State = front::filesearch::FileSearch;
    type Error = FilerError;
    type Ctrl = protocol::to_server::fscontrol::FsControl;

    fn initial_state(&self) -> Self::State {
        front::filesearch::Init(front::filesearch::Init {
            last_cache_date: None,
        })
    }

    fn name(&self) -> &str {
        "filesearch" // TODO: somehow use the exact same string as Variant::name()
    }

    async fn next(&mut self) -> Result<Self::State, Self::Error> {
        Self::next(self).await
    }

    async fn wait_until_closed(self) {
        Self::wait_until_closed(self).await
    }

    async fn quit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn control(&mut self, ctrl: Self::Ctrl) -> Result<(), Self::Error> {
        match ctrl {
            FsControl::Search(s) => self.search(s).await,
            FsControl::RefreshCache => self.refresh_cache().await,
        }
    }
}

impl HandleJobError for FilerError {
    fn is_normal_exit(&self) -> bool {
        matches!(self, FilerError::Interrupted)
    }
}
