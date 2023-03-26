use std::{
    env::temp_dir,
    fs::File,
    io,
    process::{ExitStatus, Stdio},
    time::Duration,
};
use tokio::{pin, process::Command, sync::oneshot, task::JoinHandle, time::timeout};

use crate::util::join_handle_wait;

const SIGTERM_TIMEOUT: u64 = 5;

pub type ProcResult<T> = Result<T, ProcessError>;

pub struct Process {
    exe: String,
    proc_done: Option<JoinHandle<ProcResult<ExitStatus>>>,
    kill: Option<oneshot::Sender<()>>,
}

#[derive(thiserror::Error, Debug)]
pub enum ProcessError {
    #[error(transparent)]
    IoError(#[from] io::Error),
}

impl Process {
    pub async fn oneshot(exe: String) -> ProcResult<ExitStatus> {
        Self::start(exe)?.wait().await.expect("only waited once")
    }

    pub fn start(exe: String) -> ProcResult<Self> {
        assert!(!exe.is_empty());
        // TODO: use progname in config?
        let outfile = temp_dir().join(format!("gcast_{}.stdout", exe));
        let errfile = temp_dir().join(format!("gcast_{}.stderr", exe));

        let mut child = Command::new(&exe)
            .stdin(Stdio::null())
            .stdout(File::create(outfile)?)
            .stderr(File::create(errfile)?)
            .spawn()?;

        let (kill_tx, kill_rx) = oneshot::channel();
        let pid = child.id().expect("has not been waited yet");

        log::info!("Spawned process '{}' with pid {}", exe, pid);

        let handle = tokio::spawn(async move {
            let wait = child.wait();
            pin!(wait);

            tokio::select! {
                w = &mut wait => {
                    log::debug!("Process with pid {} exited by itself", pid);
                    return Ok(w?);
                },
                _ = kill_rx => (),
            }

            log::info!("Trying to kill pid {}", pid);
            let term_succ = sigterm(pid);
            log::debug!(
                "Sent SIGTERM, success={}. Waiting {} seconds for process to exit",
                term_succ,
                SIGTERM_TIMEOUT
            );
            if let Ok(res) =
                timeout(Duration::from_secs(SIGTERM_TIMEOUT), &mut wait).await
            {
                log::debug!("Process terminated within timeout");
                return Ok(res?);
            }

            let kill_succ = sigkill(pid);
            log::debug!("Process took too long, sent SIGKILL, success={}", kill_succ);
            Ok(wait.await?)
        });

        Ok(Process {
            exe,
            proc_done: Some(handle),
            kill: Some(kill_tx),
        })
    }

    // cancel safe wait
    pub async fn wait(&mut self) -> Option<ProcResult<ExitStatus>> {
        match &mut self.proc_done {
            None => None,
            Some(pd) => {
                let res = join_handle_wait(pd).await;
                self.proc_done.take();
                Some(res)
            }
        }
    }

    pub fn kill(&mut self) -> bool {
        match self.kill.take() {
            None => false,
            Some(sender) => sender.send(()).is_ok(),
        }
    }

    pub fn name(&self) -> &str {
        &self.exe
    }
}

fn sigterm(id: u32) -> bool {
    let pid: i32 = id.try_into().expect("this should fit");
    unsafe { libc::kill(pid, libc::SIGTERM) == 0 }
}

fn sigkill(id: u32) -> bool {
    let pid: i32 = id.try_into().expect("this should fit");
    unsafe { libc::kill(pid, libc::SIGKILL) == 0 }
}
