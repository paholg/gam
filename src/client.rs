use bevy::{
    prelude::{
        Added, Assets, BuildChildren, Bundle, Commands, Component, ComputedVisibility, Entity,
        EventReader, Handle, Mesh, PbrBundle, Plugin, Query, Res, ResMut, Resource,
        StandardMaterial, Transform, Visibility, With, Without,
    },
    scene::Scene,
};
use bevy_hanabi::ParticleEffect;
use bevy_kira_audio::{
    prelude::Volume, Audio, AudioControl, AudioInstance, AudioPlugin, PlaybackState,
};
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;
use iyes_progress::ProgressPlugin;
use rand::Rng;
use tracing::info;

use crate::{
    ability::{
        grenade::{Explosion, Grenade, GrenadeKind},
        HyperSprinting, Shot, ShotHitEvent, ABILITY_Z,
    },
    Ally, AppState, DeathEvent, Enemy, Player,
};

use self::{
    asset_handler::{
        asset_handler_setup, AssetHandler, DeathEffect, HyperSprintEffect, ShotEffect,
    },
    bar::{BarPlugin, Energybar, Healthbar},
    config::{Config, ConfigPlugin},
    controls::ControlPlugin,
    splash::SplashPlugin,
};

mod asset_handler;
mod bar;
mod config;
mod controls;
mod splash;
mod ui;

const OUTLINE_DEPTH_BIAS: f32 = 0.5;

/// This plugin includes user input and graphics.
pub struct GamClientPlugin;

impl Plugin for GamClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(SplashPlugin)
            .add_plugin(
                ProgressPlugin::new(AppState::Loading)
                    .continue_to(AppState::Running)
                    .track_assets(),
            )
            .add_plugin(AudioPlugin)
            .add_plugin(ConfigPlugin)
            .add_plugin(ControlPlugin)
            .add_plugin(GraphicsPlugin)
            .insert_resource(BackgroundMusic::default())
            .add_system(background_music_system)
            .add_plugin(bevy_hanabi::HanabiPlugin);
    }
}

struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(asset_handler_setup)
            .add_plugin(InverseKinematicsPlugin)
            .add_plugin(BarPlugin)
            .add_system(draw_player_system)
            .add_system(draw_enemy_system)
            .add_system(draw_ally_system)
            .add_system(draw_shot_system)
            .add_system(draw_grenade_system)
            .add_system(draw_shot_hit_system)
            .add_system(draw_death_system)
            .add_system(draw_explosion_system)
            .add_system(draw_hyper_sprint_system)
            .add_system(draw_target_system)
            .add_system(update_target_system)
            .add_plugin(ui::UiPlugin);
    }
}

#[derive(Bundle, Default)]
struct ObjectGraphics {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

#[derive(Bundle)]
struct CharacterGraphics {
    healthbar: Healthbar,
    energybar: Energybar,
    scene: Handle<Scene>,
    outline: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

fn draw_shot_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Shot>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else { continue };
        ecmds.insert(ObjectGraphics {
            material: assets.shot.material.clone(),
            mesh: assets.shot.mesh.clone(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
        });
    }
}

fn draw_grenade_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Grenade), Added<Grenade>>,
) {
    for (entity, grenade) in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else { continue };
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
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
        });
    }
}

fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Player>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else { continue };
        ecmds.insert(CharacterGraphics {
            healthbar: Healthbar::default(),
            energybar: Energybar::default(),
            scene: assets.player.scene.clone(),
            outline: assets.player.outline_mesh.clone(),
            material: assets.player.outline_material.clone(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
        });
    }
}

fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Enemy>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else { continue };
        ecmds.insert(CharacterGraphics {
            healthbar: Healthbar::default(),
            energybar: Energybar::default(),
            scene: assets.enemy.scene.clone(),
            outline: assets.enemy.outline_mesh.clone(),
            material: assets.enemy.outline_material.clone(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
        });
    }
}

fn draw_ally_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, (Added<Ally>, Without<Player>)>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else { continue };
        ecmds.insert(CharacterGraphics {
            healthbar: Healthbar::default(),
            energybar: Energybar::default(),
            scene: assets.ally.scene.clone(),
            outline: assets.ally.outline_mesh.clone(),
            material: assets.ally.outline_material.clone(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
        });
    }
}

fn draw_shot_hit_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform), With<ShotEffect>>,
    mut event_reader: EventReader<ShotHitEvent>,
) {
    for hit in event_reader.iter() {
        let (mut effect, mut transform) = effects.get_mut(assets.shot.effect_entity).unwrap();
        *transform = hit.transform;
        effect.maybe_spawner().unwrap().reset();
        audio
            .play(assets.shot.despawn_sound.clone())
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_death_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform), With<DeathEffect>>,
    mut event_reader: EventReader<DeathEvent>,
) {
    for death in event_reader.iter() {
        let (mut effect, mut transform) = effects.get_mut(assets.player.despawn_effect).unwrap();
        *transform = death.transform;
        transform.translation.z += ABILITY_Z;
        effect.maybe_spawner().unwrap().reset();

        audio
            .play(assets.player.despawn_sound.clone())
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_explosion_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform), Without<Explosion>>,
    query: Query<(&Transform, &Explosion), Added<Explosion>>,
) {
    for (explosion_transform, explosion) in &query {
        let effect_entity = match explosion.kind {
            GrenadeKind::Frag => assets.frag_grenade.effect_entity,
            GrenadeKind::Heal => assets.heal_grenade.effect_entity,
        };
        let (mut effect, mut transform) = effects.get_mut(effect_entity).unwrap();
        *transform = *explosion_transform;
        effect.maybe_spawner().unwrap().reset();

        audio
            .play(assets.player.despawn_sound.clone())
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_hyper_sprint_system(
    assets: Res<AssetHandler>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform), With<HyperSprintEffect>>,
    query: Query<&Transform, (With<HyperSprinting>, Without<HyperSprintEffect>)>,
) {
    for sprint_transform in query.iter() {
        let (mut effect, mut transform) =
            effects.get_mut(assets.hyper_sprint.effect_entity).unwrap();
        *transform = *sprint_transform;
        effect.maybe_spawner().unwrap().reset();
    }
}

#[derive(Resource, Default)]
struct BackgroundMusic {
    name: Option<String>,
    handle: Option<Handle<AudioInstance>>,
}

fn background_music_system(
    asset_handler: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut bg_music: ResMut<BackgroundMusic>,
    assets: Res<Assets<AudioInstance>>,
) {
    let should_play = match &bg_music.handle {
        None => true,
        Some(handle) => match assets.get(handle) {
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
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..asset_handler.music.len());
        let (name, song) = asset_handler.music[idx].clone();
        info!(%name, "Playing");
        let handle = audio
            .play(song)
            .with_volume(Volume::Decibels(config.sound.music_volume))
            .handle();

        bg_music.name = Some(name);
        bg_music.handle = Some(handle);
    }
}

#[derive(Component)]
struct Target {
    owner: Entity,
}

fn draw_target_system(
    mut commands: Commands,
    query: Query<(Entity, &Player), Added<Player>>,
    asset_handler: Res<AssetHandler>,
) {
    for (entity, player) in &query {
        let target_entity = commands
            .spawn((
                PbrBundle {
                    material: asset_handler.target.material.clone(),
                    mesh: asset_handler.target.mesh.clone(),
                    transform: Transform::from_translation(player.target.extend(0.0)),
                    ..Default::default()
                },
                Target { owner: entity },
            ))
            .id();
        commands.entity(entity).push_children(&[target_entity]);
    }
}

fn update_target_system(
    player_query: Query<(&Player, &Transform)>,
    mut target_query: Query<(&mut Transform, &Target), Without<Player>>,
) {
    for (mut transform, target) in &mut target_query {
        if let Ok((player, player_transform)) = player_query.get(target.owner) {
            let rotation = player_transform.rotation.inverse();
            transform.rotation = rotation;
            transform.translation =
                rotation * (player.target.extend(0.0) - player_transform.translation);
        }
    }
}
