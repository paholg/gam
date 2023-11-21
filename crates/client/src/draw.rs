use bevy::{
    prelude::{
        Added, BuildChildren, Bundle, Commands, Entity, EventReader, Handle,
        InheritedVisibility, Mesh, PbrBundle, Plugin, Query, Res,
        StandardMaterial, Transform, Update, ViewVisibility, Visibility, With,
        Without,
    },
    scene::Scene,
};
use bevy_hanabi::EffectSpawner;
use bevy_kira_audio::{
    prelude::Volume, Audio, AudioControl,
};
use engine::{
    ability::{
        explosion::{Explosion, ExplosionKind},
        grenade::{Grenade, GrenadeKind, GrenadeLandEvent},
        seeker_rocket::SeekerRocket,
        HyperSprinting, Shot, ShotHitEvent, ABILITY_Z,
    },
    Ally, DeathEvent, Enemy, Player,
};

use crate::{
    asset_handler::{
        AssetHandler, DeathEffect, HyperSprintEffect, ShotEffect,
    },
    bar::{Energybar, Healthbar},
    Config,
};

/// A plugin for spawning graphics for newly-created entities.
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                draw_player_system,
                draw_enemy_system,
                draw_ally_system,
                draw_shot_system,
                draw_grenade_system,
                draw_grenade_outline_system,
                draw_seeker_rocket_system,
                draw_shot_hit_system,
                draw_death_system,
                draw_explosion_system,
                draw_hyper_sprint_system,
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

fn draw_seeker_rocket_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<SeekerRocket>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(ObjectGraphics {
            material: assets.seeker_rocket.material.clone(),
            mesh: assets.seeker_rocket.mesh.clone(),
            ..Default::default()
        });
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
            ExplosionKind::FragGrenade => assets.frag_grenade.effect_entity,
            ExplosionKind::HealGrenade => assets.heal_grenade.effect_entity,
            ExplosionKind::SeekerRocket => assets.seeker_rocket.effect_entity,
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
