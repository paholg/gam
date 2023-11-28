use bevy::{
    core::FrameCount,
    prelude::{
        shape, Added, AlphaMode, Assets, BuildChildren, Bundle, Color, Commands, Component, Entity,
        EventReader, Handle, InheritedVisibility, Mesh, Parent, PbrBundle, Plugin, Query, Res,
        ResMut, StandardMaterial, Transform, Update, ViewVisibility, Visibility, With, Without,
    },
    scene::Scene,
};
use bevy_hanabi::EffectSpawner;
use bevy_kira_audio::{prelude::Volume, Audio, AudioControl};
use bevy_mod_raycast::prelude::RaycastMesh;
use engine::{
    ability::{
        bullet::Bullet,
        grenade::{Grenade, GrenadeKind, GrenadeLandEvent},
        seeker_rocket::SeekerRocket,
        HyperSprinting,
    },
    level::Floor,
    lifecycle::DeathEvent,
    Ally, Enemy, Kind, Player,
};

use crate::{
    asset_handler::AssetHandler,
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
                raycast_scene_system,
                draw_player_system,
                draw_enemy_system,
                draw_ally_system,
                draw_shot_system,
                draw_grenade_system,
                draw_grenade_outline_system,
                draw_seeker_rocket_system,
                draw_death_system,
                draw_hyper_sprint_system,
                draw_wall_system,
            ),
        );
    }
}

/// A Component to be added to entities with `Scene`s, that we want to act as
/// `RaycastMesh<()>`s.
#[derive(Component, Default)]
struct RaycastScene;

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
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    raycast: RaycastScene,
}

fn raycast_scene_system(
    mut commands: Commands,
    q_meshes: Query<(Entity, &Parent), Added<Handle<Mesh>>>,
    q_children: Query<&Parent>,
    q_parents: Query<&Handle<Scene>, With<RaycastScene>>,
) {
    // A `Scene` has meshes as its grandchildren, so we need a silly bit of
    // indirection to tell if we should add our `RaycastMesh`.
    for (entity, parent) in q_meshes.iter() {
        let Ok(parent) = q_children.get(parent.get()) else {
            continue;
        };
        let Ok(grandparent) = q_children.get(parent.get()) else {
            continue;
        };
        if q_parents.get(grandparent.get()).is_ok() {
            commands.entity(entity).insert(RaycastMesh::<()>::default());
        }
    }
}

fn draw_shot_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Bullet>>,
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
        let outline = commands
            .spawn(PbrBundle {
                mesh: assets.player.outline_mesh.clone(),
                material: assets.player.outline_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.01),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(CharacterGraphics {
                scene: assets.player.scene.clone(),
                ..Default::default()
            })
            .push_children(&[outline]);
    }
}

fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Enemy>>,
) {
    for entity in query.iter() {
        let outline = commands
            .spawn(PbrBundle {
                mesh: assets.enemy.outline_mesh.clone(),
                material: assets.enemy.outline_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.01),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert((CharacterGraphics {
                scene: assets.enemy.scene.clone(),
                ..Default::default()
            },))
            .push_children(&[outline]);
    }
}

fn draw_ally_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, (Added<Ally>, Without<Player>)>,
) {
    for entity in query.iter() {
        let outline = commands
            .spawn(PbrBundle {
                mesh: assets.ally.outline_mesh.clone(),
                material: assets.ally.outline_material.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.01),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(CharacterGraphics {
                scene: assets.ally.scene.clone(),
                ..Default::default()
            })
            .push_children(&[outline]);
    }
}

fn draw_death_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    mut event_reader: EventReader<DeathEvent>,
    frame: Res<FrameCount>,
) {
    // TODO: Reset
    for death in event_reader.read() {
        let effect = match death.kind {
            Kind::Other => continue,
            Kind::Player => &mut assets.player.despawn_effect,
            Kind::Enemy => &mut assets.enemy.despawn_effect,
            Kind::Ally => &mut assets.ally.despawn_effect,
            Kind::Bullet => &mut assets.shot.collision_effect,
            Kind::FragGrenade => &mut assets.frag_grenade.explosion_effect,
            Kind::HealGrenade => &mut assets.heal_grenade.explosion_effect,
            Kind::SeekerRocket => &mut assets.seeker_rocket.explosion_effect,
        };

        effect.trigger(&mut commands, death.transform, &mut effects, &frame);

        let sound = match death.kind {
            Kind::Bullet => assets.shot.despawn_sound.clone(),
            Kind::Other => continue,
            Kind::Player
            | Kind::Enemy
            | Kind::Ally
            | Kind::FragGrenade
            | Kind::HealGrenade
            | Kind::SeekerRocket => assets.player.despawn_sound.clone(),
        };

        audio
            .play(sound)
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_hyper_sprint_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    query: Query<&Transform, (With<HyperSprinting>, Without<EffectSpawner>)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.hyper_sprint.effect;

    for &sprint_transform in query.iter() {
        effect.trigger(&mut commands, sprint_transform, &mut effects, &frame);
    }
}

fn draw_wall_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Floor), Added<Floor>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.6, 0.8, 0.2, 0.8),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    });

    for (entity, floor) in &query {
        let shape = shape::Box {
            min_x: -floor.dim.x * 0.5,
            max_x: floor.dim.x * 0.5,
            min_y: -floor.dim.y * 0.5,
            max_y: floor.dim.y * 0.5,
            min_z: -floor.dim.z * 0.5,
            max_z: floor.dim.z * 0.5,
        };

        // First add InheritedVisibility to our entity to make bevy happy.
        commands
            .entity(entity)
            .insert(InheritedVisibility::default());

        let wall = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(shape.into()),
                    material: material.clone(),
                    ..Default::default()
                },
                RaycastMesh::<()>::default(),
            ))
            .id();
        commands.entity(entity).push_children(&[wall]);
    }
}
