use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use libmpv::{AsyncHandle, Event, Handle, LogLevel};
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("invalid usage".into());
    }

    let filepath: PathBuf = args.remove(1).into();

    inner_main(&filepath).await?;
    Ok(())
}

async fn inner_main(file: &Path) -> libmpv::Result<()> {
    let mut handle = Handle::new()?;
    handle.request_log_messages(LogLevel::Info)?;
    handle.read_config_file()?;

    let handle = handle.init()?;
    let mut handle = AsyncHandle::new(handle);

    handle.enable_default_bindings()?;

    handle.observe_paused()?;
    handle.observe_playback_time()?;
    handle.observe_media_title()?;
    handle.observe_track_list()?;
    handle.loadfile(file)?;
    handle.set_idle(libmpv::Idle::No)?;

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    loop {
        tokio::select! {
            event = handle.wait_event_async() => {
                println!("main: {event:#?}");
                if let Event::Shutdown = event {
                    break;
                }
            }
            Ok(_) = stdin.next_line() => {
                handle.toggle_pause()?;
            }
        }
    }

    println!("main quit");

    Ok(())
}
