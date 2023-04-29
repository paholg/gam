use bevy::{
    prelude::{
        Added, Audio, Bundle, Commands, ComputedVisibility, Entity, EventReader, Handle, Mesh,
        PlaybackSettings, Plugin, Query, Res, StandardMaterial, Transform, Visibility, With,
        Without,
    },
    scene::Scene,
};
use bevy_hanabi::ParticleEffect;
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;

use crate::{
    ability::{HyperSprinting, Shot, ShotHitEvent, ABILITY_Z},
    Ally, DeathEvent, Enemy, Player,
};

use self::{
    asset_handler::{
        asset_handler_setup, AssetHandler, DeathEffect, HyperSprintEffect, ShotEffect,
    },
    config::{Config, ConfigPlugin},
    controls::ControlPlugin,
    healthbar::{Healthbar, HealthbarPlugin},
};

mod asset_handler;
mod config;
mod controls;
mod healthbar;
mod ui;

const OUTLINE_DEPTH_BIAS: f32 = 0.5;

/// This plugin includes user input and graphics.
pub struct GamClientPlugin;

impl Plugin for GamClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ConfigPlugin)
            .add_plugin(ControlPlugin)
            .add_plugin(GraphicsPlugin)
            .add_plugin(bevy_hanabi::HanabiPlugin);
    }
}

struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(asset_handler_setup)
            .add_plugin(InverseKinematicsPlugin)
            .add_plugin(HealthbarPlugin)
            .add_system(draw_player_system)
            .add_system(draw_enemy_system)
            .add_system(draw_ally_system)
            .add_system(draw_shot_system)
            .add_system(draw_shot_hit_system)
            .add_system(draw_death_system)
            .add_system(draw_hyper_sprint_system)
            .add_plugin(ui::UiPlugin);
    }
}

#[derive(Bundle)]
struct ObjectGraphics {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
}

#[derive(Bundle)]
struct CharacterGraphics {
    healthbar: Healthbar,
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

fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Player>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else { continue };
        ecmds.insert(CharacterGraphics {
            healthbar: Healthbar::default(),
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
    player: Query<&Transform, (With<Player>, Without<ShotEffect>)>,
) {
    let player = player
        .get_single()
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    for hit in event_reader.iter() {
        let (mut effect, mut transform) = effects.get_mut(assets.shot.effect_entity).unwrap();
        *transform = hit.transform;
        effect.maybe_spawner().unwrap().reset();
        audio.play_spatial_with_settings(
            assets.shot.despawn_sound.clone(),
            PlaybackSettings {
                repeat: false,
                volume: config.sound.effects_volume * 0.5,
                speed: 1.0,
            },
            player,
            1.0,
            hit.transform.translation,
        );

        // FIXME: There is currently a bug where killing the last enemy with
        // multiple shots causes the new enemies to spawn and then teleport to
        // NaN. To prevent this, we only play one shot hit sound per frame.
        break;
    }
}

fn draw_death_system(
    assets: Res<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform), With<DeathEffect>>,
    mut event_reader: EventReader<DeathEvent>,
    player: Query<&Transform, (With<Player>, Without<DeathEffect>)>,
) {
    let player = player
        .get_single()
        .map(ToOwned::to_owned)
        .unwrap_or_default();
    for death in event_reader.iter() {
        let (mut effect, mut transform) = effects.get_mut(assets.player.despawn_effect).unwrap();
        *transform = death.transform;
        transform.translation.z += ABILITY_Z;
        effect.maybe_spawner().unwrap().reset();

        audio.play_spatial_with_settings(
            assets.player.despawn_sound.clone(),
            PlaybackSettings {
                repeat: false,
                volume: config.sound.effects_volume,
                speed: 1.0,
            },
            player,
            1.0,
            death.transform.translation,
        );
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

fn background_music_system() {}
