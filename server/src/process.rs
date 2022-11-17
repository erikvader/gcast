use std::{
    env::temp_dir,
    fs::File,
    io::Result as IOResult,
    process::{ExitStatus, Stdio},
    time::Duration,
};
use tokio::{pin, process::Command, sync::oneshot, task::JoinHandle, time::timeout};

use crate::util::join_handle_wait;

const SIGTERM_TIMEOUT: u64 = 5;

pub struct Process {
    exe: String,
    proc_done: JoinHandle<IOResult<ExitStatus>>,
    kill: Option<oneshot::Sender<()>>,
}

impl Process {
    pub fn start(exe: String) -> IOResult<Self> {
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
                    return w;
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
                return res;
            }

            let kill_succ = sigkill(pid);
            log::debug!("Process took too long, sent SIGKILL, success={}", kill_succ);
            wait.await
        });

        Ok(Process {
            exe,
            proc_done: handle,
            kill: Some(kill_tx),
        })
    }

    // cancel safe wait
    pub async fn wait(&mut self) -> IOResult<ExitStatus> {
        join_handle_wait(&mut self.proc_done).await
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
