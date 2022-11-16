use std::{fs, io, path::PathBuf, sync::LazyLock};

use bevy::prelude::KeyCode;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::error;

// TODO: NAME THESE THINGS
const ORG: &str = "Paho Corp";
const NAME: &str = "Gam";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Home directory not found from system. Cannot save or load config.")]
    HomeDirNotFound,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Config parse error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("Config save error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

/// Return the path to the config file, if able.
/// Creates any necessary directories of they do not exist.
fn config_file() -> Result<PathBuf, Error> {
    let proj_dirs = ProjectDirs::from("", ORG, NAME).ok_or(Error::HomeDirNotFound)?;
    let config_dir = proj_dirs.config_dir();

    fs::create_dir_all(&config_dir)?;

    let mut path = config_dir.to_owned();
    path.push("config.toml");
    Ok(path)
}

fn load_config() -> Result<Config, Error> {
    let contents = fs::read_to_string(config_file()?)?;
    let config = toml::de::from_str(&contents)?;
    Ok(config)
}

pub fn save_config() -> Result<(), Error> {
    let config = toml::ser::to_string(LazyLock::force(&CONFIG))?;
    fs::write(&config_file()?, &config)?;

    Ok(())
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| match load_config() {
    Ok(config) => config,
    Err(error) => {
        error!(%error, "Error loading config");
        Config::default()
    }
});

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub controls: Controls,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Controls {
    pub left: KeyCode,
    pub right: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,

    pub ability1: KeyCode,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            left: KeyCode::A,
            right: KeyCode::D,
            up: KeyCode::W,
            down: KeyCode::S,

            ability1: KeyCode::Space,
        }
    }
}
