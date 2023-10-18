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
use engine::{ability::Ability, Player};
use leafwing_input_manager::{
    prelude::{DualAxis, InputManagerPlugin, InputMap, VirtualDPad},
    Actionlike, InputManagerBundle,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

// TODO: NAME THESE THINGS
const ORG: &str = "Paho Corp";
const NAME: &str = "Gam";

pub const ABILITY_COUNT: usize = 5;

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
            .add_plugins(InputManagerPlugin::<Action>::default())
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

#[derive(Debug, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct Config {
    pub controls: InputMap<Action>,
    pub graphics: Graphics,
    pub sound: Sound,
    pub player: PlayerAbilities,
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

fn default_controls() -> InputMap<Action> {
    let mut map = InputMap::default();
    map.insert(KeyCode::Escape, Action::Menu)
        .insert(GamepadButtonType::Start, Action::Menu)
        .insert(DualAxis::left_stick(), Action::Move)
        .insert(
            VirtualDPad {
                up: GamepadButtonType::DPadUp.into(),
                down: GamepadButtonType::DPadUp.into(),
                left: GamepadButtonType::DPadLeft.into(),
                right: GamepadButtonType::DPadRight.into(),
            },
            Action::Move,
        )
        .insert(
            VirtualDPad {
                up: KeyCode::W.into(),
                down: KeyCode::S.into(),
                left: KeyCode::A.into(),
                right: KeyCode::D.into(),
            },
            Action::Move,
        )
        .insert(DualAxis::right_stick(), Action::Aim)
        .insert(MouseButton::Left, Action::Ability0)
        .insert(GamepadButtonType::RightTrigger2, Action::Ability0)
        .insert(MouseButton::Right, Action::Ability1)
        .insert(GamepadButtonType::LeftTrigger2, Action::Ability1)
        .insert(KeyCode::Space, Action::Ability2)
        .insert(GamepadButtonType::South, Action::Ability2)
        .insert(KeyCode::E, Action::Ability3)
        .insert(MouseButton::Other(8), Action::Ability3)
        .insert(GamepadButtonType::RightTrigger, Action::Ability3)
        .insert(KeyCode::Q, Action::Ability4)
        .insert(MouseButton::Other(9), Action::Ability4)
        .insert(GamepadButtonType::LeftTrigger, Action::Ability4);
    map
}

impl Config {
    fn load() -> Result<Self, Error> {
        let contents = fs::read_to_string(config_file()?)?;
        let config = serde_json::from_str(&contents)?;
        info!("Config loaded: {:#?}", config);
        Ok(config)
    }

    fn save(&self) -> Result<(), Error> {
        let config = serde_json::to_string_pretty(self)?;
        fs::write(config_file()?, config)?;

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
    pub effects_volume: f64,
    pub music_volume: f64,
    pub speech_volume: f64,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            effects_volume: -20.0,
            music_volume: -20.0,
            speech_volume: -20.0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerAbilities {
    pub abilities: [Ability; ABILITY_COUNT],
}

impl Default for PlayerAbilities {
    fn default() -> Self {
        Self {
            abilities: [
                Ability::Shoot,
                Ability::Shotgun,
                Ability::HyperSprint,
                Ability::HealGrenade,
                Ability::FragGrenade,
            ],
        }
    }
}

#[derive(
    Actionlike,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    TypePath,
)]
pub enum Action {
    Ability0 = 0,
    Ability1 = 1,
    Ability2 = 2,
    Ability3 = 3,
    Ability4 = 4,
    Move,
    Aim,
    Menu,
}

fn spawn_input_manager(
    mut commands: Commands,
    query: Query<Entity, Added<Player>>,
    config: Res<Config>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::<Action> {
                input_map: config.controls.clone(),
                ..Default::default()
            });
    }
}
