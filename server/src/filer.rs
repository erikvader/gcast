use std::sync::atomic::AtomicBool;

use tokio::sync::{mpsc, oneshot};

static FILER_THREAD_ON: AtomicBool = AtomicBool::new(false);

pub type FilerResult<T> = Result<T, FilerError>;

#[derive(Debug, thiserror::Error)]
#[error("hej")]
pub enum FilerError {}

pub type Command = (); // TODO:
type StateRcv = mpsc::Receiver<FilerResult<FilerState>>;
type StateSnd = mpsc::Sender<FilerResult<FilerState>>;
type HandleResp = oneshot::Sender<FilerResult<()>>;
type HandleSnd = mpsc::Sender<(Command, HandleResp)>;

struct FilerState {}
