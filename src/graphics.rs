use bevy::{
    prelude::{
        Added, Bundle, Commands, ComputedVisibility, Entity, EventReader, Handle, Mesh, Plugin,
        Query, Res, StandardMaterial, Transform, Visibility, With, Without,
    },
    scene::Scene,
};
use bevy_hanabi::ParticleEffect;
use bevy_mod_inverse_kinematics::InverseKinematicsPlugin;

use crate::{
    ability::{HyperSprinting, Shot, ShotHitEvent},
    Ally, Enemy, Player,
};

use self::{
    asset_handler::{asset_handler_setup, AssetHandler, HyperSprintEffect, ShotEffect},
    healthbar::{Healthbar, HealthbarPlugin},
};

mod asset_handler;
mod healthbar;

pub struct GraphicsPlugin;

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
            .add_system(draw_hyper_sprint_system);
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
    mut effects: Query<(&mut ParticleEffect, &mut Transform), With<ShotEffect>>,
    mut event_reader: EventReader<ShotHitEvent>,
) {
    for hit in event_reader.iter() {
        let (mut effect, mut transform) = effects.get_mut(assets.shot.effect_entity).unwrap();
        *transform = hit.transform;
        effect.maybe_spawner().unwrap().reset();
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
