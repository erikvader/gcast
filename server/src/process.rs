use std::{
    env::temp_dir,
    fs::File,
    io::Result as IOResult,
    process::{ExitStatus, Stdio},
    time::Duration,
};
use tokio::{
    pin,
    process::Command,
    sync::{mpsc, oneshot},
    time::timeout,
};

const SIGTERM_TIMEOUT: u64 = 5;

pub struct Process {
    proc_done: mpsc::Receiver<IOResult<ExitStatus>>,
    kill: Option<oneshot::Sender<()>>,
}

impl Process {
    pub fn start(exe: &str) -> IOResult<Self> {
        assert!(!exe.is_empty());
        let outfile = temp_dir().join(format!("gcast_{}.stdout", exe));
        let errfile = temp_dir().join(format!("gcast_{}.stderr", exe));

        let mut child = Command::new(exe)
            .stdin(Stdio::null())
            .stdout(File::create(outfile)?)
            .stderr(File::create(errfile)?)
            .spawn()?;

        let (done_tx, done_rx) = mpsc::channel(1);
        let (kill_tx, kill_rx) = oneshot::channel();
        let pid = child.id().expect("has not been waited yet");

        log::info!("Spawned process '{}' with pid {}", exe, pid);

        tokio::spawn(async move {
            let wait = child.wait();
            pin!(wait);

            tokio::select! {
                w = &mut wait => {
                    log::debug!("Process with pid {} exited by itself", pid);
                    done_tx.send(w).await.ok();
                    return;
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
                done_tx.send(res).await.ok();
                return;
            }

            let kill_succ = sigkill(pid);
            log::debug!("Sending SIGKILL, success={}", kill_succ);
            done_tx.send(wait.await).await.ok();
        });

        Ok(Process {
            proc_done: done_rx,
            kill: Some(kill_tx),
        })
    }

    // cancel safe wait
    pub async fn wait(&mut self) -> IOResult<ExitStatus> {
        if let Some(msg) = self.proc_done.recv().await {
            msg
        } else {
            panic!("Process task must have panicked, or this was called twice");
        }
    }

    pub fn kill(&mut self) -> bool {
        match self.kill.take() {
            None => false,
            Some(sender) => sender.send(()).is_ok(),
        }
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
