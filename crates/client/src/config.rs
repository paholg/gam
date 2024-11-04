use std::{fs, io, path::PathBuf};

use bevy::{
    core_pipeline::fxaa,
    prelude::{
        Added, Commands, Entity, GamepadButtonType, KeyCode, MouseButton, Plugin, Query, Res,
        Resource, Update,
    },
    reflect::Reflect,
};
// use bevy_ui_navigation::{events::ScopeDirection, prelude::NavRequest};
use directories::ProjectDirs;
use engine::{multiplayer::Action, player::AbilityIds, Player};
use leafwing_input_manager::{
    prelude::{
        GamepadStick, GamepadVirtualDPad, InputManagerPlugin, InputMap, KeyboardVirtualDPad,
    },
    Actionlike, InputManagerBundle,
};
use serde::{Deserialize, Serialize};
use subenum::subenum;
use tracing::{error, info};

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
    map.insert(UserAction::Menu, KeyCode::Escape)
        .insert(UserAction::Menu, GamepadButtonType::Start)
        .insert_dual_axis(UserAction::Move, GamepadStick::LEFT)
        .insert_dual_axis(UserAction::Move, GamepadVirtualDPad::DPAD)
        .insert_dual_axis(
            UserAction::Move,
            KeyboardVirtualDPad::new(
                KeyCode::KeyE.into(),
                KeyCode::KeyD.into(),
                KeyCode::KeyS.into(),
                KeyCode::KeyF.into(),
            ),
        )
        .insert_dual_axis(UserAction::Aim, GamepadStick::RIGHT)
        // Left arm
        .insert(UserAction::LeftArm, MouseButton::Left)
        .insert(UserAction::LeftArm, GamepadButtonType::LeftTrigger2)
        .insert(UserAction::LeftArmSecondary, KeyCode::KeyT)
        .insert(UserAction::LeftArmSecondary, GamepadButtonType::West)
        // Right arm
        .insert(UserAction::RightArm, MouseButton::Right)
        .insert(UserAction::RightArm, GamepadButtonType::RightTrigger2)
        .insert(UserAction::RightArmSecondary, KeyCode::KeyV)
        .insert(UserAction::RightArmSecondary, GamepadButtonType::East)
        // Left Shoulder
        .insert(UserAction::LeftShoulder, KeyCode::KeyW)
        .insert(UserAction::LeftShoulder, MouseButton::Other(8))
        .insert(UserAction::LeftShoulder, GamepadButtonType::LeftTrigger)
        // Right Shoulder
        .insert(UserAction::RightShoulder, KeyCode::KeyR)
        .insert(UserAction::RightShoulder, MouseButton::Other(9))
        .insert(UserAction::RightShoulder, GamepadButtonType::RightTrigger)
        // Legs
        .insert(UserAction::Legs, KeyCode::Space)
        .insert(UserAction::Legs, GamepadButtonType::South)
        // Head
        .insert(UserAction::Head, KeyCode::KeyV)
        .insert(UserAction::Head, GamepadButtonType::North)
        // Menu controls
        .insert(UserAction::TabLeft, GamepadButtonType::LeftTrigger)
        .insert(UserAction::TabRight, GamepadButtonType::RightTrigger)
        .insert(UserAction::TabLeft, GamepadButtonType::LeftTrigger2)
        .insert(UserAction::TabRight, GamepadButtonType::RightTrigger2)
        .insert(UserAction::Select, GamepadButtonType::South)
        .insert(UserAction::Cancel, GamepadButtonType::East)
        .insert(UserAction::TabLeft, KeyCode::KeyW)
        .insert(UserAction::TabRight, KeyCode::KeyR)
        .insert(UserAction::Select, KeyCode::Enter)
        .insert(UserAction::Select, MouseButton::Left)
        .insert(UserAction::Cancel, MouseButton::Right);
    map
}

impl Config {
    // TODO: We'll load config once we can change it in-game.
    #[allow(unused)]
    fn load() -> Result<Self, Error> {
        let path = config_file()?;
        let contents = fs::read_to_string(&path)?;
        let config = serde_json::from_str(&contents)?;
        info!("Config loaded from {}", path.display());
        Ok(config)
    }

    #[allow(unused)]
    fn save(&self) -> Result<(), Error> {
        let config = serde_json::to_string_pretty(self)?;
        let path = config_file()?;
        fs::write(&path, config)?;
        info!("Config written to {}", path.display());

        Ok(())
    }

    pub fn new() -> Config {
        Config::default()
        // match Self::load() {
        //     Ok(config) => config,
        //     Err(error) => {
        //         warn!(?error, "Couldn't load config; using default");
        //         let config = Config::default();
        //         if let Err(error) = config.save() {
        //             warn!(?error, "Couldn't save new config!");
        //         }
        //         config
        //     }
        // }
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
    pub ability_ids: AbilityIds,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            ability_ids: AbilityIds {
                left_arm: "gun".into(),
                right_arm: "gun".into(),
                left_shoulder: "frag_grenade".into(),
                right_shoulder: "gravity_ball".into(),
                legs: "noop".into(),
                head: "noop".into(),
            },
        }
    }
}

#[derive(
    Actionlike,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Reflect,
    Serialize,
)]
#[subenum(GameAction, MenuAction)]
pub enum UserAction {
    // Game actions
    #[subenum(GameAction)]
    LeftArm,
    #[subenum(GameAction)]
    LeftArmSecondary,
    #[subenum(GameAction)]
    RightArm,
    #[subenum(GameAction)]
    RightArmSecondary,
    #[subenum(GameAction)]
    LeftShoulder,
    #[subenum(GameAction)]
    RightShoulder,
    #[subenum(GameAction)]
    Legs,
    #[subenum(GameAction)]
    Head,

    // Not real actions; just indicate that an AxisPair was used.
    #[actionlike(DualAxis)]
    Move,
    #[actionlike(DualAxis)]
    Aim,

    // This one's weird, as it is a game action, we just handle it specially.
    Menu,

    // Menu actions
    #[subenum(MenuAction)]
    Select,
    #[subenum(MenuAction)]
    Cancel,
    #[subenum(MenuAction)]
    TabLeft,
    #[subenum(MenuAction)]
    TabRight,
}

impl From<GameAction> for Action {
    fn from(value: GameAction) -> Self {
        match value {
            GameAction::LeftArm => Action::LeftArm,
            GameAction::LeftArmSecondary => Action::LeftArmSecondary,
            GameAction::RightArm => Action::RightArm,
            GameAction::RightArmSecondary => Action::RightArmSecondary,
            GameAction::LeftShoulder => Action::LeftShoulder,
            GameAction::RightShoulder => Action::RightShoulder,
            GameAction::Legs => Action::Legs,
            GameAction::Head => Action::Head,
        }
    }
}

// TODO: bevy-ui-navigation
// impl From<MenuAction> for NavRequest {
//     fn from(value: MenuAction) -> Self {
//         match value {
//             MenuAction::Select => NavRequest::Action,
//             MenuAction::Cancel => NavRequest::Cancel,
//             MenuAction::TabLeft =>
// NavRequest::ScopeMove(ScopeDirection::Previous),
// MenuAction::TabRight => NavRequest::ScopeMove(ScopeDirection::Next),
//         }
//     }
// }

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
