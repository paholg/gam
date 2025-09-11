use std::time::Duration;

use ability::AbilityPlugin;
use aim::AimPlugin;
use bar::BarPlugin;
use bevy::{
    asset::{AssetServer, LoadedFolder},
    ecs::system::{Commands, Query},
    prelude::{Assets, Handle, Plugin, Res, ResMut, Resource, Startup, Transform, Update, Vec3},
    state::state::{OnEnter, OnExit},
    window::{CursorGrabMode, Window},
};
use bevy_framepace::FramepaceSettings;
use bevy_kira_audio::{
    prelude::Volume, Audio, AudioControl, AudioInstance, AudioPlugin, PlaybackState,
};
use config::ConfigPlugin;
use draw::DrawPlugin;
use engine::{time::TIMESTEP, AppState, UP};
use rand::Rng;
use splash::SplashPlugin;

pub mod ability;
mod aim;
mod bar;
mod camera;
pub mod color_gradient;
mod config;
mod controls;
pub mod debug;
pub mod draw;
mod i18n;
mod particles;
mod shapes;
mod splash;
mod ui;
mod world;

pub use config::Config;
pub use controls::ControlPlugin;

use crate::camera::CameraPlugin;

/// Return a Transform such that things normally in the XY-plane will instead be
/// correctly oriented in the XZ plane.
pub fn in_plane() -> Transform {
    Transform::IDENTITY.looking_to(-UP, Vec3::Z)
}

/// This plugin includes user input and graphics.
pub struct GamClientPlugin;

impl Plugin for GamClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            SplashPlugin,
            AudioPlugin,
            ConfigPlugin,
            GraphicsPlugin,
            AbilityPlugin,
            CameraPlugin,
            bevy_hanabi::HanabiPlugin,
            bevy_framepace::FramepacePlugin,
        ))
        .add_systems(Startup, (load_music, setup_framepace))
        .add_systems(Update, background_music_system)
        .add_systems(Startup, world::setup)
        .add_systems(OnEnter(AppState::Running), hide_cursor)
        .add_systems(OnExit(AppState::Running), show_cursor);
    }
}

struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((BarPlugin, ui::UiPlugin, DrawPlugin, AimPlugin));
    }
}

#[derive(Resource)]
struct BackgroundMusic {
    name: Option<String>,
    handle: Option<Handle<AudioInstance>>,
    folder: Handle<LoadedFolder>,
}

impl BackgroundMusic {
    fn new(asset_server: &AssetServer) -> Self {
        let folder = asset_server.load_folder("third-party/audio/Galacti-Chrons Weird Music Pack");
        Self {
            name: None,
            handle: None,
            folder,
        }
    }
}

fn load_music(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(BackgroundMusic::new(&assets));
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    // FIXME: Remove
    settings.limiter = bevy_framepace::Limiter::Manual(Duration::from_secs_f32(TIMESTEP));
}

fn background_music_system(
    audio: Res<Audio>,
    config: Res<Config>,
    mut bg_music: ResMut<BackgroundMusic>,
    audio_assets: Res<Assets<AudioInstance>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
) {
    let should_play = match &bg_music.handle {
        None => true,
        Some(handle) => match audio_assets.get(handle) {
            Some(asset) => asset.state() == PlaybackState::Stopped,
            None => false,
        },
    };

    if should_play {
        if let Some(folder) = loaded_folders.get(&bg_music.folder) {
            let mut rng = rand::rng();
            let idx = rng.random_range(0..folder.handles.len());
            let track = folder.handles[idx].clone().typed();
            let name = track
                .path()
                .unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let handle = audio
                .play(track)
                .with_volume(Volume::Decibels(config.audio.music_volume))
                .handle();

            bg_music.name = Some(name);
            bg_music.handle = Some(handle);
        }
    }
}

// #[derive(Debug)]
// struct Hierarchy {
//     #[allow(dead_code)]
//     entity: Entity,
//     #[allow(dead_code)]
//     components: Vec<String>,
//     #[allow(dead_code)]
//     children: Vec<Hierarchy>,
// }

// /// Print the full hierarchy that includes this entity.
// pub fn print_hierarchy(
//     initial_entity: Entity,
//     world: &World,
//     q_parents: Query<&Children>,
//     q_children: Query<&ChildOf>,
// ) {
//     // First let's go to the top
//     let mut entity = initial_entity;
//     while let Ok(child_of) = q_children.get(entity) {
//         entity = child_of.parent();
//     }

//     let hierarchy = print_hierarchy_inner(entity, world, &q_parents);

//     println!("**************************************************");
//     println!("{:#?}", hierarchy);
//     println!("**************************************************");
// }

// fn print_hierarchy_inner(entity: Entity, world: &World, q_parents: &Query<&Children>) -> Hierarchy {
//     let components = world
//         .inspect_entity(entity)
//         .map(ComponentInfo::name)
//         .map(ToOwned::to_owned)
//         .collect();

//     let children = q_parents
//         .get(entity)
//         .map(|children| {
//             children
//                 .iter()
//                 .map(|child| print_hierarchy_inner(*child, world, q_parents))
//                 .collect()
//         })
//         .unwrap_or_default();

//     Hierarchy {
//         entity,
//         components,
//         children,
//     }
// }

fn hide_cursor(mut windows: Query<&mut Window>) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    window.cursor_options.visible = false;
    window.cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn show_cursor(mut windows: Query<&mut Window>) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    window.cursor_options.visible = true;
    window.cursor_options.grab_mode = CursorGrabMode::None;
}
