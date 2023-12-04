use aim::AimPlugin;
use bevy::{
    asset::LoadedFolder,
    ecs::component::ComponentInfo,
    prelude::{
        Assets, Children, Entity, Handle, Parent, Plugin, Query, Res, ResMut, Resource, Startup,
        Transform, Update, Vec3, World,
    },
};

use bevy_kira_audio::{
    prelude::Volume, Audio, AudioControl, AudioInstance, AudioPlugin, PlaybackState,
};
use draw::DrawPlugin;
use iyes_progress::ProgressPlugin;
use rand::Rng;

use engine::{AppState, UP};

use self::{
    asset_handler::{asset_handler_setup, AssetHandler},
    bar::BarPlugin,
    config::ConfigPlugin,
    splash::SplashPlugin,
};

mod aim;
mod asset_handler;
mod bar;
pub mod color_gradient;
mod config;
mod controls;
mod draw;
mod particles;
mod shapes;
mod splash;
mod ui;
mod world;

pub use config::Config;
pub use controls::ControlPlugin;
pub use draw::draw_pathfinding_system;

rust_i18n::i18n!("../../locales");

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 12.0, 12.0);

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
            ProgressPlugin::new(AppState::Loading)
                .continue_to(AppState::Running)
                .track_assets(),
            AudioPlugin,
            ConfigPlugin,
            GraphicsPlugin,
            bevy_hanabi::HanabiPlugin,
        ))
        .insert_resource(BackgroundMusic::default())
        .add_systems(Update, background_music_system)
        .add_systems(Startup, world::setup);
    }
}

struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, asset_handler_setup).add_plugins((
            BarPlugin,
            ui::UiPlugin,
            DrawPlugin,
            AimPlugin,
        ));
    }
}

#[derive(Resource, Default)]
struct BackgroundMusic {
    name: Option<String>,
    handle: Option<Handle<AudioInstance>>,
}

fn background_music_system(
    assets: Res<AssetHandler>,
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
        if let Some(folder) = loaded_folders.get(&assets.music) {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..folder.handles.len());
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
                .with_volume(Volume::Decibels(config.sound.music_volume))
                .handle();

            bg_music.name = Some(name);
            bg_music.handle = Some(handle);
        }
    }
}

#[derive(Debug)]
struct Hierarchy {
    #[allow(dead_code)]
    entity: Entity,
    #[allow(dead_code)]
    components: Vec<String>,
    #[allow(dead_code)]
    children: Vec<Hierarchy>,
}

/// Print the full hierarchy that includes this entity.
pub fn print_hierarchy(
    initial_entity: Entity,
    world: &World,
    q_parents: Query<&Children>,
    q_children: Query<&Parent>,
) {
    // First let's go to the top
    let mut entity = initial_entity;
    while let Ok(parent) = q_children.get(entity) {
        entity = parent.get();
    }

    let hierarchy = print_hierarchy_inner(entity, world, &q_parents);

    println!("**************************************************");
    println!("{:#?}", hierarchy);
    println!("**************************************************");
}

fn print_hierarchy_inner(entity: Entity, world: &World, q_parents: &Query<&Children>) -> Hierarchy {
    let components = world
        .inspect_entity(entity)
        .into_iter()
        .map(ComponentInfo::name)
        .map(ToOwned::to_owned)
        .collect();

    let children = q_parents
        .get(entity)
        .map(|children| {
            children
                .iter()
                .map(|child| print_hierarchy_inner(*child, world, q_parents))
                .collect()
        })
        .unwrap_or_default();

    Hierarchy {
        entity,
        components,
        children,
    }
}
