use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context;
use tokio::sync::OnceCell;

pub const PROGNAME: &str = "gcast";
pub const CONFIG_NAME: &str = "config.toml";

static CONF: OnceCell<Config> = OnceCell::const_new();

#[derive(Debug, serde::Deserialize)]
struct Config {
    root_dirs: Vec<String>,
    port: u16,
    mpv: Mpv,
    spotify: Spotify,
}

#[derive(Debug, serde::Deserialize)]
struct Spotify {
    executable: String,
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

// TODO: use
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

// TODO: use
pub fn spotify_exe() -> &'static str {
    &get_instance().spotify.executable
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

fn read_root_dirs(path: &Path) -> io::Result<Vec<String>> {
    Ok(fs::read_to_string(path)?
        .lines()
        // NOTE: only specifying a '/' is not supported
        .map(|s| s.trim_end_matches("/").to_string())
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
