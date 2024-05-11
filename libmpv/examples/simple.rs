use std::{
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
};

use libmpv::{Event, Handle, LogLevel};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("invalid usage".into());
    }

    let filepath: PathBuf = args.remove(1).into();

    inner_main(&filepath)?;
    Ok(())
}

fn inner_main(file: &Path) -> libmpv::Result<()> {
    let mut handle = Handle::new()?;
    handle.request_log_messages(LogLevel::Info)?;
    handle.read_config_file()?;

    let mut handle = handle.init()?;
    let version = handle.version()?;
    println!("{version}");

    handle.enable_default_bindings()?;

    handle.observe_paused()?;
    handle.observe_playback_time()?;
    handle.observe_media_title()?;
    handle.observe_track_list()?;
    handle.loadfile(file)?;
    handle.set_idle(libmpv::Idle::No)?;

    loop {
        let event = handle.wait_event_infinite();
        println!("main: {event:#?}");
        if let Event::Shutdown = event {
            break;
        }
    }

    println!("main quit");

    Ok(())
}
