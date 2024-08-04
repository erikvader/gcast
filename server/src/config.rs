use std::{fs, path::PathBuf};

use anyhow::Context;
use tokio::sync::OnceCell;

pub const PROGNAME: &str = "gcast";
const CONFIG_NAME: &str = "config.toml";

static CONF: OnceCell<Config> = OnceCell::const_new();

// TODO: only make public in state_machine? Or should every module ever be able to read
// from here?
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    root_dirs: Vec<String>,
    port: u16,
    poweroff_exe: String,
    refresh_cache_boot: bool,
    spotify: Spotify,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Spotify {
    executable: String,
    fullscreen_exe: String,
}

pub fn init_config() -> anyhow::Result<()> {
    let conf_file = conf_dir().join(CONFIG_NAME);
    let conts = fs::read_to_string(&conf_file)
        .with_context(|| format!("reading config file at {:?}", conf_file))?;

    let conf: Config = toml::from_str(&conts).context("parsing config file as TOML")?;

    CONF.set(conf).context("setting the global conf variable")?;
    Ok(())
}

fn get_instance() -> &'static Config {
    CONF.get().expect("Config was not initialized")
}

pub fn root_dirs() -> &'static [String] {
    &get_instance().root_dirs
}

pub fn port() -> u16 {
    get_instance().port
}

pub fn spotify_exe() -> &'static str {
    &get_instance().spotify.executable
}

pub fn spotify_fullscreen_exe() -> &'static str {
    &get_instance().spotify.fullscreen_exe
}

pub fn poweroff_exe() -> &'static str {
    &get_instance().poweroff_exe
}

pub fn refresh_cache_boot() -> bool {
    get_instance().refresh_cache_boot
}

// TODO: make configurable
pub fn mpv_conf_dir() -> PathBuf {
    conf_dir().join("mpv")
}

pub fn conf_dir() -> PathBuf {
    dirs::config_dir()
        .expect("could not get config dir")
        .join(PROGNAME)
}

pub fn cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("could not get cache dir")
        .join(PROGNAME)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_default_config() {
        let conts = include_str!("../config.def.toml");
        let conf: Result<Config, _> = toml::from_str(&conts);
        println!("conf: {conf:#?}");
        assert!(conf.is_ok());
    }
}
