use std::fs;
use std::io;
use std::path::PathBuf;

use bevy::core_pipeline::fxaa;
use bevy::prelude::Added;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::GamepadButton;
use bevy::prelude::KeyCode;
use bevy::prelude::MouseButton;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Resource;
use bevy::prelude::Update;
use bevy::reflect::Reflect;
use directories::ProjectDirs;
use engine::multiplayer::Action;
use engine::player::AbilityIds;
use engine::Player;
use leafwing_input_manager::prelude::GamepadStick;
use leafwing_input_manager::prelude::InputManagerPlugin;
use leafwing_input_manager::prelude::InputMap;
use leafwing_input_manager::prelude::VirtualDPad;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::InputManagerBundle;
use serde::Deserialize;
use serde::Serialize;
use subenum::subenum;
use tracing::error;
use tracing::info;

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
    pub audio: Audio,
    pub player: PlayerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            controls: default_controls(),
            graphics: Default::default(),
            audio: Default::default(),
            player: Default::default(),
        }
    }
}

fn default_controls() -> InputMap<UserAction> {
    let mut map = InputMap::default();
    map.insert(UserAction::Menu, KeyCode::Escape)
        .insert(UserAction::Menu, GamepadButton::Start)
        .insert_dual_axis(UserAction::Move, GamepadStick::LEFT)
        .insert_dual_axis(UserAction::Move, VirtualDPad::dpad())
        .insert_dual_axis(
            UserAction::Move,
            VirtualDPad::new(KeyCode::KeyE, KeyCode::KeyD, KeyCode::KeyS, KeyCode::KeyF),
        )
        .insert_dual_axis(UserAction::Aim, GamepadStick::RIGHT)
        // Left Arm
        .insert(UserAction::LeftArm, MouseButton::Left)
        .insert(UserAction::LeftArm, GamepadButton::LeftTrigger2)
        .insert(UserAction::LeftArmSecondary, KeyCode::KeyW)
        .insert(UserAction::LeftArmSecondary, GamepadButton::West)
        // Right Arm
        .insert(UserAction::RightArm, MouseButton::Right)
        .insert(UserAction::RightArm, GamepadButton::RightTrigger2)
        .insert(UserAction::RightArmSecondary, KeyCode::KeyE)
        .insert(UserAction::RightArmSecondary, GamepadButton::East)
        // Left Shoulder
        .insert(UserAction::LeftShoulder, MouseButton::Back)
        .insert(UserAction::LeftShoulder, GamepadButton::LeftTrigger)
        // Right Shoulder
        .insert(UserAction::RightShoulder, MouseButton::Forward)
        .insert(UserAction::RightShoulder, GamepadButton::RightTrigger)
        // Legs
        .insert(UserAction::Legs, KeyCode::Space)
        .insert(UserAction::Legs, GamepadButton::South)
        // Head
        .insert(UserAction::Head, KeyCode::KeyG)
        .insert(UserAction::Head, GamepadButton::North)
        // Menu controls
        .insert(UserAction::TabLeft, GamepadButton::LeftTrigger)
        .insert(UserAction::TabLeft, GamepadButton::LeftTrigger2)
        .insert(UserAction::TabLeft, KeyCode::KeyW)
        .insert(UserAction::TabRight, GamepadButton::RightTrigger)
        .insert(UserAction::TabRight, GamepadButton::RightTrigger2)
        .insert(UserAction::TabRight, KeyCode::KeyR)
        .insert(UserAction::Select, GamepadButton::South)
        .insert(UserAction::Select, KeyCode::Enter)
        .insert(UserAction::Select, KeyCode::Space)
        .insert(UserAction::Select, MouseButton::Left)
        .insert(UserAction::Cancel, GamepadButton::East)
        .insert(UserAction::Cancel, KeyCode::Escape)
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
#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum MsaaSamples {
    One = 1,
    #[default]
    Four = 4,
}

#[derive(Debug, Default, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
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
pub struct Audio {
    pub global_volume: f64,
    pub effects_volume: f64,
    pub music_volume: f64,
    pub speech_volume: f64,
}

impl Default for Audio {
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
                right_arm: "rocket".into(),
                left_shoulder: "frag_grenade".into(),
                right_shoulder: "gravity_ball".into(),
                legs: "transport_beam".into(),
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
