#![feature(path_file_prefix)]

use aim::AimPlugin;
use bevy::{
    asset::LoadedFolder,
    prelude::{Assets, Handle, Plugin, Res, ResMut, Resource, Startup, Update, Vec3},
};

use bevy_kira_audio::{
    prelude::Volume, Audio, AudioControl, AudioInstance, AudioPlugin, PlaybackState,
};
use draw::DrawPlugin;
use iyes_progress::ProgressPlugin;
use rand::Rng;

use engine::AppState;

use self::{
    asset_handler::{asset_handler_setup, AssetHandler},
    bar::BarPlugin,
    config::ConfigPlugin,
    splash::SplashPlugin,
};

mod aim;
mod asset_handler;
mod bar;
mod config;
mod controls;
mod draw;
mod shapes;
mod splash;
mod ui;
mod world;

pub use config::Config;
pub use controls::ControlPlugin;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

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
            Some(asset) => {
                if asset.state() == PlaybackState::Stopped {
                    true
                } else {
                    false
                }
            }
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
                .file_prefix()
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
