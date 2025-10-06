use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets, Handle, LoadedFolder},
    audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume},
    ecs::{
        component::Component,
        entity::Entity,
        query::Without,
        system::{Commands, Res, Single},
    },
    transform::components::Transform,
};
use rand::Rng;
use tracing::info;

use crate::Config;

pub fn play_sound_effect(
    commands: &mut Commands,
    config: &Config,
    sound: Handle<AudioSource>,
    transform: Transform,
) {
    commands.spawn((
        AudioPlayer::new(sound),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Despawn,
            volume: Volume::Decibels(config.audio.effects_volume),
            spatial: true,
            ..Default::default()
        },
        transform,
    ));
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, load_music)
            .add_systems(Update, play_music);
    }
}

#[derive(Component)]
struct Music {
    name: Option<String>,
    folder: Handle<LoadedFolder>,
}

impl Music {
    fn new(asset_server: &AssetServer) -> Self {
        let folder = asset_server.load_folder("third-party/audio/Galacti-Chrons Weird Music Pack");
        Self { name: None, folder }
    }
}

fn load_music(mut commands: Commands, assets: Res<AssetServer>) {
    info!("loading music");
    commands.spawn(Music::new(&assets));
}

fn play_music(
    mut commands: Commands,
    music: Option<Single<(Entity, &mut Music), Without<AudioPlayer>>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    config: Res<Config>,
) {
    let Some(mut music) = music else {
        return;
    };
    let (entity, ref mut music) = *music;
    let Some(folder) = loaded_folders.get(&music.folder) else {
        return;
    };

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
    commands.entity(entity).insert((
        AudioPlayer::new(track),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Remove,
            volume: Volume::Decibels(config.audio.music_volume),
            spatial: false,
            ..Default::default()
        },
    ));
    music.name = Some(name);
}
