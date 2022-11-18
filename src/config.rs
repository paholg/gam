use std::{fs, io, path::PathBuf, sync::LazyLock};

use bevy::{
    core_pipeline::fxaa,
    prelude::{GamepadButton, Input, KeyCode, MouseButton, Res},
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::ability::Ability;

// TODO: NAME THESE THINGS
const ORG: &str = "Paho Corp";
const NAME: &str = "Gam";

const ABILITY_COUNT: usize = 3;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Home directory not found from system. Cannot save or load config.")]
    HomeDirNotFound,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    // #[error("Config parse error: {0}")]
    // TomlDe(#[from] toml::de::Error),
    // #[error("Config save error: {0}")]
    // TomlSer(#[from] toml::ser::Error),
    #[error("Config parse/serialize error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Return the path to the config file, if able.
/// Creates any necessary directories of they do not exist.
// TODO: The toml crate does not currently support enums. SAD. So we're using
//  json.
fn config_file() -> Result<PathBuf, Error> {
    let proj_dirs = ProjectDirs::from("", ORG, NAME).ok_or(Error::HomeDirNotFound)?;
    let config_dir = proj_dirs.config_dir();

    fs::create_dir_all(&config_dir)?;

    let mut path = config_dir.to_owned();
    path.push("config.json");
    Ok(path)
}

fn load_config() -> Result<Config, Error> {
    let contents = fs::read_to_string(config_file()?)?;
    let config = serde_json::from_str(&contents)?;
    info!("Config loaded: {:#?}", config);
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Error> {
    let config = serde_json::to_string_pretty(config)?;
    fs::write(&config_file()?, &config)?;

    Ok(())
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let config = match load_config() {
        Ok(config) => config,
        Err(error) => {
            error!(%error, "Error loading config");
            Config::default()
        }
    };
    // TODO: For now, we always save config on load to pickup changes. Stop
    // doing this once we can edit the config in-game.
    if let Err(error) = save_config(&config) {
        error!(?error, "Error saving config");
    }
    config
});

pub fn config() -> &'static Config {
    LazyLock::force(&CONFIG)
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub controls: Controls,
    pub graphics: Graphics,
    pub player: Player,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Button {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    Controller(GamepadButton),
}

impl Button {
    pub fn pressed(
        &self,
        keyboard_input: &Res<Input<KeyCode>>,
        mouse_input: &Res<Input<MouseButton>>,
    ) -> bool {
        match self {
            Button::Keyboard(button) => keyboard_input.pressed(*button),
            Button::Mouse(button) => mouse_input.pressed(*button),
            Button::Controller(_) => todo!(),
        }
    }

    pub fn just_pressed(
        &self,
        keyboard_input: &Res<Input<KeyCode>>,
        mouse_input: &Res<Input<MouseButton>>,
    ) -> bool {
        match self {
            Button::Keyboard(button) => keyboard_input.just_pressed(*button),
            Button::Mouse(button) => mouse_input.just_pressed(*button),
            Button::Controller(_) => todo!(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Controls {
    pub left: Button,
    pub right: Button,
    pub up: Button,
    pub down: Button,

    pub abilities: [Button; ABILITY_COUNT],

    pub menu: Button,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            left: Button::Keyboard(KeyCode::A),
            right: Button::Keyboard(KeyCode::D),
            up: Button::Keyboard(KeyCode::W),
            down: Button::Keyboard(KeyCode::S),

            abilities: [
                Button::Mouse(MouseButton::Left),
                Button::Mouse(MouseButton::Right),
                Button::Keyboard(KeyCode::Space),
            ],

            menu: Button::Keyboard(KeyCode::Escape),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Graphics {
    pub anti_aliasing: AntiAliasing,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum AntiAliasing {
    #[default]
    None,
    Fxaa {
        sensitivity: Sensitivity,
    },
    Msaa {
        samples: MsaaSamples,
    },
}

/// Presently, only 1 or 4 samples is supported
/// See https://github.com/gfx-rs/wgpu/issues/1832
#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone)]
pub enum MsaaSamples {
    One = 1,
    #[default]
    Four = 4,
}

#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone)]
pub enum Sensitivity {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
    Extreme,
}

impl From<Sensitivity> for fxaa::Sensitivity {
    fn from(value: Sensitivity) -> Self {
        match value {
            Sensitivity::Low => Self::Low,
            Sensitivity::Medium => Self::Medium,
            Sensitivity::High => Self::High,
            Sensitivity::Ultra => Self::Ultra,
            Sensitivity::Extreme => Self::Extreme,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub abilities: [Ability; ABILITY_COUNT],
}

impl Default for Player {
    fn default() -> Self {
        Self {
            abilities: [Ability::Shoot, Ability::None, Ability::HyperSprint],
        }
    }
}
