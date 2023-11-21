use bevy::prelude::{
    Added, BuildChildren, Commands, Component, Entity, Parent, PbrBundle, Plugin, Query, Res,
    SpatialBundle, Transform, Update, Vec3, With, Without,
};
use bevy_mod_raycast::prelude::{DeferredRaycastingPlugin, RaycastSource};
use engine::{ability::ABILITY_Z, Player, Target};

use crate::asset_handler::AssetHandler;

/// A plugin for managing aiming, such as drawing and updating the cursor.
pub struct AimPlugin;

impl Plugin for AimPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(DeferredRaycastingPlugin::<()>::default())
            .add_systems(
                Update,
                (
                    draw_target_system,
                    update_target_system,
                    draw_laser_system,
                    update_laser_system,
                ),
            );
    }
}

#[derive(Component)]
struct CursorTarget;

fn draw_target_system(
    mut commands: Commands,
    query: Query<(Entity, &Target), Added<Player>>,
    asset_handler: Res<AssetHandler>,
) {
    for (entity, target) in &query {
        let target_entity = commands
            .spawn((
                PbrBundle {
                    material: asset_handler.target.cursor_material.clone(),
                    mesh: asset_handler.target.cursor_mesh.clone(),
                    transform: Transform::from_translation(target.0.extend(0.0)),
                    ..Default::default()
                },
                CursorTarget,
            ))
            .id();
        commands.entity(entity).push_children(&[target_entity]);
    }
}

fn update_target_system(
    player_query: Query<(&Transform, &Target), With<Player>>,
    mut target_query: Query<(&Parent, &mut Transform), (Without<Player>, With<CursorTarget>)>,
) {
    for (parent, mut transform) in &mut target_query {
        if let Ok((player_transform, target)) = player_query.get(parent.get()) {
            let rotation = player_transform.rotation.inverse();
            transform.rotation = rotation;
            transform.translation =
                rotation * (target.0.extend(0.0) - player_transform.translation);
        }
    }
}

#[derive(Component)]
struct LaserSight {
    raycast: Entity,
}

fn draw_laser_system(
    mut commands: Commands,
    query: Query<Entity, Added<Player>>,
    asset_handler: Res<AssetHandler>,
) {
    for entity in &query {
        let mut laser_transform = Transform::from_translation(ABILITY_Z * Vec3::Z);
        update_laser(asset_handler.target.laser_length, &mut laser_transform);

        let raycast = commands
            .spawn((
                RaycastSource::<()>::new_transform_empty(),
                SpatialBundle {
                    transform: Transform::from_translation(ABILITY_Z * Vec3::Z)
                        .looking_to(Vec3::Y, Vec3::Z),
                    ..Default::default()
                },
            ))
            .id();
        let laser = commands
            .spawn((
                PbrBundle {
                    material: asset_handler.target.laser_material.clone(),
                    mesh: asset_handler.target.laser_mesh.clone(),
                    transform: laser_transform,
                    ..Default::default()
                },
                LaserSight { raycast },
            ))
            .id();

        commands.entity(entity).push_children(&[laser, raycast]);
    }
}

fn update_laser_system(
    mut laser_query: Query<(&mut Transform, &LaserSight)>,
    raycast_query: Query<&RaycastSource<()>, Without<LaserSight>>,
    asset_handler: Res<AssetHandler>,
) {
    for (mut transform, laser_sight) in &mut laser_query {
        let Ok(raycast) = raycast_query.get(laser_sight.raycast) else {
            tracing::warn!("LaserSight is missing its Raycast!");
            return;
        };
        let len = match raycast.get_nearest_intersection() {
            Some((_entity, data)) => data.distance(),
            None => asset_handler.target.laser_length,
        };
        update_laser(len, &mut transform);
    }
}

fn update_laser(length: f32, transform: &mut Transform) {
    transform.scale.y = length;
    transform.translation.y = length * 0.5;
}
