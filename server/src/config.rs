use std::{fs, path::PathBuf};

use anyhow::Context;
use tokio::sync::OnceCell;

pub const PROGNAME: &str = "gcast";
const CONFIG_NAME: &str = "config.toml";

static CONF: OnceCell<Config> = OnceCell::const_new();

// TODO: only make public in state_machine?
#[derive(Debug, serde::Deserialize)]
struct Config {
    root_dirs: Vec<String>,
    port: u16,
    poweroff_exe: String,
    mpv: Mpv,
    spotify: Spotify,
}

#[derive(Debug, serde::Deserialize)]
struct Spotify {
    executable: String,
    fullscreen_exe: String,
}

#[derive(Debug, serde::Deserialize)]
struct Mpv {
    properties: toml::value::Table,
}

pub fn init_config() -> anyhow::Result<()> {
    let conf_file = conf_dir().join(CONFIG_NAME);
    let conts = fs::read_to_string(&conf_file)
        .with_context(|| format!("reading config file at {:?}", conf_file))?;

    let conf: Config = toml::from_str(&conts).context("parsing config file as TOML")?;

    if conf.mpv.properties.iter().any(|(_, v)| !v.is_str()) {
        anyhow::bail!("Mpv values must be strings");
    }

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

pub fn mpv_options() -> Vec<(String, String)> {
    get_instance()
        .mpv
        .properties
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                v.as_str()
                    .expect("has been checked while reading the config")
                    .to_string(),
            )
        })
        .collect()
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
