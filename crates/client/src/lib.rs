#![feature(path_file_prefix)]

use bevy::{
    asset::LoadedFolder,
    prelude::{
        Added, Assets, BuildChildren, Bundle, Commands, Component, Entity, EventReader, Handle,
        InheritedVisibility, Mesh, PbrBundle, Plugin, Query, Res, ResMut, Resource,
        StandardMaterial, Startup, Transform, Update, Vec3, ViewVisibility, Visibility, With,
        Without,
    },
    scene::Scene,
};
use bevy_hanabi::EffectSpawner;
use bevy_kira_audio::{
    prelude::Volume, Audio, AudioControl, AudioInstance, AudioPlugin, PlaybackState,
};
use iyes_progress::ProgressPlugin;
use rand::Rng;

use engine::{
    ability::{
        grenade::{Explosion, Grenade, GrenadeKind, GrenadeLandEvent},
        HyperSprinting, Shot, ShotHitEvent, ABILITY_Z,
    },
    Ally, AppState, DeathEvent, Enemy, Player, Target,
};

use self::{
    asset_handler::{
        asset_handler_setup, AssetHandler, DeathEffect, HyperSprintEffect, ShotEffect,
    },
    bar::{BarPlugin, Energybar, Healthbar},
    config::ConfigPlugin,
    splash::SplashPlugin,
};

mod asset_handler;
mod bar;
mod config;
mod controls;
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
        app.add_systems(Startup, asset_handler_setup)
            .add_plugins((BarPlugin, ui::UiPlugin))
            .add_systems(
                Update,
                (
                    draw_player_system,
                    draw_enemy_system,
                    draw_ally_system,
                    draw_shot_system,
                    draw_grenade_system,
                    draw_grenade_outline_system,
                    draw_shot_hit_system,
                    draw_death_system,
                    draw_explosion_system,
                    draw_hyper_sprint_system,
                    draw_target_system,
                    update_target_system,
                ),
            );
    }
}

#[derive(Bundle, Default)]
struct ObjectGraphics {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
struct CharacterGraphics {
    healthbar: Healthbar,
    energybar: Energybar,
    scene: Handle<Scene>,
    outline: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

fn draw_shot_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Shot>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(ObjectGraphics {
            material: assets.shot.material.clone(),
            mesh: assets.shot.mesh.clone(),
            ..Default::default()
        });
    }
}

fn draw_grenade_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Grenade), Added<Grenade>>,
) {
    for (entity, grenade) in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        let (mesh, material) = match grenade.kind {
            GrenadeKind::Frag => (
                assets.frag_grenade.mesh.clone(),
                assets.frag_grenade.material.clone(),
            ),
            GrenadeKind::Heal => (
                assets.heal_grenade.mesh.clone(),
                assets.heal_grenade.material.clone(),
            ),
        };
        ecmds.insert(ObjectGraphics {
            material,
            mesh,
            ..Default::default()
        });
    }
}

fn draw_grenade_outline_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<&Grenade>,
    mut event_reader: EventReader<GrenadeLandEvent>,
) {
    for event in event_reader.read() {
        let entity = event.entity;
        let Ok(grenade) = query.get(entity) else {
            tracing::warn!(?entity, "Can't find grenade to outline.");
            continue;
        };
        let (mesh, material) = match grenade.kind {
            GrenadeKind::Frag => (
                assets.frag_grenade.outline_mesh.clone(),
                assets.frag_grenade.outline_material.clone(),
            ),
            GrenadeKind::Heal => (
                assets.heal_grenade.outline_mesh.clone(),
                assets.heal_grenade.outline_material.clone(),
            ),
        };
        let outline_entity = commands
            .spawn(PbrBundle {
                mesh,
                material,
                ..Default::default()
            })
            .id();
        commands.entity(entity).push_children(&[outline_entity]);
    }
}

fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Player>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(CharacterGraphics {
            scene: assets.player.scene.clone(),
            outline: assets.player.outline_mesh.clone(),
            material: assets.player.outline_material.clone(),
            ..Default::default()
        });
    }
}

fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Enemy>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(CharacterGraphics {
            scene: assets.enemy.scene.clone(),
            outline: assets.enemy.outline_mesh.clone(),
            material: assets.enemy.outline_material.clone(),
            ..Default::default()
        });
    }
}

fn draw_ally_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, (Added<Ally>, Without<Player>)>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(CharacterGraphics {
            scene: assets.ally.scene.clone(),
            outline: assets.ally.outline_mesh.clone(),
            material: assets.ally.outline_material.clone(),
            ..Default::default()
        });
    }
}

fn draw_shot_hit_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner), With<ShotEffect>>,
    mut event_reader: EventReader<ShotHitEvent>,
) {
    for hit in event_reader.read() {
        let Ok((mut transform, mut effect_spawner)) = effects.get_mut(assets.shot.effect_entity)
        else {
            tracing::warn!(?hit, "Could not get shot effect");
            continue;
        };
        *transform = hit.transform;
        effect_spawner.reset();
        audio
            .play(assets.shot.despawn_sound.clone())
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_death_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner), With<DeathEffect>>,
    mut event_reader: EventReader<DeathEvent>,
) {
    for death in event_reader.read() {
        let Ok((mut transform, mut effect_spawner)) = effects.get_mut(assets.player.despawn_effect)
        else {
            tracing::warn!(?death, "Could not get death effect");
            continue;
        };
        *transform = death.transform;
        transform.translation.z += ABILITY_Z;
        effect_spawner.reset();

        audio
            .play(assets.player.despawn_sound.clone())
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_explosion_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner), Without<Explosion>>,
    query: Query<(&Transform, &Explosion), Added<Explosion>>,
) {
    for (explosion_transform, explosion) in &query {
        let effect_entity = match explosion.kind {
            GrenadeKind::Frag => assets.frag_grenade.effect_entity,
            GrenadeKind::Heal => assets.heal_grenade.effect_entity,
        };
        let Ok((mut transform, mut effect_spawner)) = effects.get_mut(effect_entity) else {
            tracing::warn!(
                ?explosion_transform,
                ?explosion,
                "Could not get effect for explosion."
            );
            continue;
        };
        *transform = *explosion_transform;
        effect_spawner.reset();

        audio
            .play(assets.player.despawn_sound.clone())
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_hyper_sprint_system(
    assets: Res<AssetHandler>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner), With<HyperSprintEffect>>,
    query: Query<&Transform, (With<HyperSprinting>, Without<HyperSprintEffect>)>,
) {
    for sprint_transform in query.iter() {
        let Ok((mut transform, mut effect_spawner)) =
            effects.get_mut(assets.hyper_sprint.effect_entity)
        else {
            tracing::warn!(?sprint_transform, "Could not get sprint effect.");
            continue;
        };
        *transform = *sprint_transform;
        effect_spawner.reset();
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

#[derive(Component)]
struct CursorTarget {
    owner: Entity,
}

fn draw_target_system(
    mut commands: Commands,
    query: Query<(Entity, &Target), Added<Player>>,
    asset_handler: Res<AssetHandler>,
) {
    for (entity, target) in &query {
        let target_entity = commands
            .spawn((
                PbrBundle {
                    material: asset_handler.target.material.clone(),
                    mesh: asset_handler.target.mesh.clone(),
                    transform: Transform::from_translation(target.0.extend(0.0)),
                    ..Default::default()
                },
                CursorTarget { owner: entity },
            ))
            .id();
        commands.entity(entity).push_children(&[target_entity]);
    }
}

fn update_target_system(
    player_query: Query<(&Transform, &Target), With<Player>>,
    mut target_query: Query<(&mut Transform, &CursorTarget), Without<Player>>,
) {
    for (mut transform, target) in &mut target_query {
        if let Ok((player_transform, target)) = player_query.get(target.owner) {
            let rotation = player_transform.rotation.inverse();
            transform.rotation = rotation;
            transform.translation =
                rotation * (target.0.extend(0.0) - player_transform.translation);
        }
    }
}
