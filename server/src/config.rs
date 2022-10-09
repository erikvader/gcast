use std::{
    fs, io,
    path::{Path, PathBuf},
};

use tokio::sync::OnceCell;

pub const ROOT_DIRS_FILENAME: &str = "root_dirs";
pub const PROGNAME: &str = "gcast";

static CONF: OnceCell<Config> = OnceCell::const_new();

#[derive(Debug)]
struct Config {
    root_dirs: Vec<String>,
    conf_dir: PathBuf,
    cache_dir: PathBuf,
}

pub fn init_config() {
    let conf_dir = dirs::config_dir()
        .expect("could not get config dir")
        .join(PROGNAME);
    let cache_dir = dirs::cache_dir()
        .expect("could not get cache dir")
        .join(PROGNAME);

    let root_dirs = match read_root_dirs(&conf_dir.join(ROOT_DIRS_FILENAME)) {
        Ok(dirs) => dirs,
        Err(e) => {
            log::error!("Failed to read root dirs config file: '{}'", e);
            Vec::new()
        }
    };

    CONF.set(Config {
        root_dirs,
        conf_dir,
        cache_dir,
    })
    .expect("Failed to init config");
}

fn get_instance() -> &'static Config {
    CONF.get().expect("Config was not initialized")
}

pub fn root_dirs() -> &'static [String] {
    &get_instance().root_dirs
}

pub fn conf_dir() -> &'static Path {
    &get_instance().conf_dir
}

pub fn cache_dir() -> &'static Path {
    &get_instance().cache_dir
}

fn read_root_dirs(path: &Path) -> io::Result<Vec<String>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .map(|s| s.to_string())
        .filter_map(|s| {
            if !s.starts_with("/") {
                log::error!("Root dir path '{}' is not absolute, ignoring", s);
                None
            } else {
                Some(s)
            }
        })
        .collect())
}
