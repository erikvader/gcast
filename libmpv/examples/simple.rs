use std::{error::Error, ffi::OsString, path::PathBuf, thread, time::Duration};

use libmpv::{AudioDriver, Event, Format, MpvHandle, Property};

pub fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("invalid usage".into());
    }

    let filepath: PathBuf = args.remove(1).into();

    inner_main(filepath)?;
    Ok(())
}

fn inner_main(file: PathBuf) -> libmpv::Result<()> {
    let mut handle = MpvHandle::new()?;
    handle.set_audio_driver(AudioDriver::Pulse)?;
    let mut handle = handle.init()?;
    let version = handle.version()?;
    println!("{}", version);

    handle.enable_default_bindings()?;

    handle.observe_property(Property::Pause, Format::String)?;
    handle.loadfile(file)?;

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
