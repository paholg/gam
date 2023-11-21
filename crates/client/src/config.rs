use std::{fs, io, path::PathBuf};

use bevy::{
    core_pipeline::fxaa,
    prelude::{
        Added, Commands, Entity, GamepadButtonType, KeyCode, MouseButton, Plugin, Query, Res,
        Resource, Update,
    },
    reflect::TypePath,
};
use directories::ProjectDirs;
use engine::{
    ability::{Abilities, Ability},
    Player,
};
use leafwing_input_manager::{
    prelude::{DualAxis, InputManagerPlugin, InputMap, VirtualDPad},
    Actionlike, InputManagerBundle,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

// TODO: NAME THESE THINGS
const ORG: &str = "Paho Corp";
const NAME: &str = "Gam";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Home directory not found from system. Cannot save or load config.")]
    HomeDirNotFound,
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Config parse/serialize error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(Config::new())
            .add_plugins(InputManagerPlugin::<UserAction>::default())
            .add_systems(Update, spawn_input_manager);
    }
}

pub fn project_dirs() -> Result<ProjectDirs, Error> {
    ProjectDirs::from("", ORG, NAME).ok_or(Error::HomeDirNotFound)
}

/// Return the path to the config file, if able.
/// Creates any necessary directories of they do not exist.
fn config_file() -> Result<PathBuf, Error> {
    let proj_dirs = project_dirs()?;
    let config_dir = proj_dirs.config_dir();

    fs::create_dir_all(config_dir)?;

    let mut path = config_dir.to_owned();
    path.push("config.json");
    Ok(path)
}

/// Persistent Config that should remain set between game sessions.
// TODO: On config change, reload the relevant entities.
#[derive(Debug, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct Config {
    pub controls: InputMap<UserAction>,
    pub graphics: Graphics,
    pub sound: Sound,
    pub player: PlayerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            controls: default_controls(),
            graphics: Default::default(),
            sound: Default::default(),
            player: Default::default(),
        }
    }
}

fn default_controls() -> InputMap<UserAction> {
    let mut map = InputMap::default();
    map.insert(KeyCode::Escape, UserAction::Menu)
        .insert(GamepadButtonType::Start, UserAction::Menu)
        .insert(DualAxis::left_stick(), UserAction::Move)
        .insert(
            VirtualDPad {
                up: GamepadButtonType::DPadUp.into(),
                down: GamepadButtonType::DPadUp.into(),
                left: GamepadButtonType::DPadLeft.into(),
                right: GamepadButtonType::DPadRight.into(),
            },
            UserAction::Move,
        )
        .insert(
            VirtualDPad {
                up: KeyCode::E.into(),
                down: KeyCode::D.into(),
                left: KeyCode::S.into(),
                right: KeyCode::F.into(),
            },
            UserAction::Move,
        )
        .insert(DualAxis::right_stick(), UserAction::Aim)
        .insert(MouseButton::Left, UserAction::Ability0)
        .insert(GamepadButtonType::RightTrigger2, UserAction::Ability0)
        .insert(MouseButton::Right, UserAction::Ability1)
        .insert(GamepadButtonType::LeftTrigger2, UserAction::Ability1)
        .insert(KeyCode::Space, UserAction::Ability2)
        .insert(GamepadButtonType::South, UserAction::Ability2)
        .insert(KeyCode::W, UserAction::Ability3)
        .insert(MouseButton::Other(8), UserAction::Ability3)
        .insert(GamepadButtonType::RightTrigger, UserAction::Ability3)
        .insert(KeyCode::R, UserAction::Ability4)
        .insert(MouseButton::Other(9), UserAction::Ability4)
        .insert(GamepadButtonType::LeftTrigger, UserAction::Ability4);
    map
}

impl Config {
    fn load() -> Result<Self, Error> {
        let path = config_file()?;
        let contents = fs::read_to_string(&path)?;
        let config = serde_json::from_str(&contents)?;
        info!("Config loaded from {}", path.display());
        Ok(config)
    }

    fn save(&self) -> Result<(), Error> {
        let config = serde_json::to_string_pretty(self)?;
        let path = config_file()?;
        fs::write(&path, config)?;
        info!("Config written to {}", path.display());

        Ok(())
    }

    pub fn new() -> Config {
        match Self::load() {
            Ok(config) => config,
            Err(error) => {
                warn!(?error, "Couldn't load config; using default");
                let config = Config::default();
                if let Err(error) = config.save() {
                    warn!(?error, "Couldn't save new config!");
                }
                config
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Graphics {
    pub anti_aliasing: AntiAliasing,
    pub bloom: bool,
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
pub struct Sound {
    pub global_volume: f64,
    pub effects_volume: f64,
    pub music_volume: f64,
    pub speech_volume: f64,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            global_volume: -20.0,
            effects_volume: -20.0,
            music_volume: -20.0,
            speech_volume: -20.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub abilities: Abilities,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        let abilities = Abilities::new(vec![
            Ability::Gun,
            Ability::SeekerRocket,
            Ability::HyperSprint,
            Ability::HealGrenade,
            Ability::FragGrenade,
        ]);
        Self { abilities }
    }
}

#[derive(
    Debug,
    TypePath,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Actionlike,
)]
pub enum UserAction {
    Ability0,
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Menu,

    Move,
    Aim,
}

fn spawn_input_manager(
    mut commands: Commands,
    query: Query<Entity, Added<Player>>,
    config: Res<Config>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::<UserAction> {
                input_map: config.controls.clone(),
                ..Default::default()
            });
    }
}
