use bevy::hierarchy::Children;
use bevy::hierarchy::HierarchyQueryExt;
use bevy::math::Dir3;
use bevy::math::Ray3d;
use bevy::pbr::MeshMaterial3d;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Mesh3d;
use bevy::prelude::MeshRayCast;
use bevy::prelude::Parent;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::RayCastSettings;
use bevy::prelude::Res;
use bevy::prelude::Transform;
use bevy::prelude::Update;
use bevy::prelude::With;
use bevy::prelude::Without;
use bevy::scene::SceneInstance;
use engine::AbilityOffset;
use engine::Player;
use engine::Target;
use engine::To2d;
use engine::To3d;

use crate::asset_handler::AssetHandler;
use crate::in_plane;

/// A plugin for managing aiming, such as drawing and updating the cursor.
pub struct AimPlugin;

impl Plugin for AimPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                draw_target_system,
                update_target_system,
                draw_laser_system,
                update_laser_system,
                scene_block_sight_system,
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
                Mesh3d(asset_handler.target.cursor_mesh.clone_weak()),
                MeshMaterial3d(asset_handler.target.cursor_material.clone_weak()),
                in_plane().with_translation(target.0.to_3d(0.0)),
                NotShadowCaster,
                NotShadowReceiver,
                CursorTarget,
            ))
            .id();
        commands.entity(entity).add_children(&[target_entity]);
    }
}

fn update_target_system(
    player_query: Query<(&Transform, &Target), With<Player>>,
    mut target_query: Query<(&Parent, &mut Transform), (Without<Player>, With<CursorTarget>)>,
) {
    for (parent, mut transform) in &mut target_query {
        if let Ok((player_transform, target)) = player_query.get(parent.get()) {
            let mut t = in_plane();
            let rotation = player_transform.rotation.inverse();
            t.rotate(rotation);
            t.translation = rotation * (target.0.to_3d(0.01) - player_transform.translation);

            *transform = t;
        }
    }
}

#[derive(Component)]
pub struct BlocksSight;

#[derive(Component)]
struct LaserSight;

fn draw_laser_system(
    mut commands: Commands,
    query: Query<(Entity, &AbilityOffset), Added<Player>>,
    asset_handler: Res<AssetHandler>,
) {
    for (entity, ability_offset) in &query {
        let laser_transform = in_plane().with_translation(ability_offset.to_vec());

        let laser = commands
            .spawn((
                Mesh3d(asset_handler.target.laser_mesh.clone_weak()),
                MeshMaterial3d(asset_handler.target.laser_material.clone_weak()),
                laser_transform,
                NotShadowCaster,
                NotShadowReceiver,
                LaserSight {},
            ))
            .id();

        commands.entity(entity).add_children(&[laser]);
    }
}

fn update_laser_system(
    mut raycast: MeshRayCast,
    asset_handler: Res<AssetHandler>,
    mut laser_query: Query<
        (&Parent, &mut Transform),
        (With<LaserSight>, Without<Player>, Without<BlocksSight>),
    >,
    player_query: Query<(&Transform, &Target), With<Player>>,
    blocks_sight_query: Query<(), With<BlocksSight>>,
) {
    let filter = |entity| blocks_sight_query.get(entity).is_ok();
    let settings = RayCastSettings::default().with_filter(&filter);
    for (parent, mut transform) in &mut laser_query {
        let (player_transform, target) = player_query.get(parent.get()).expect("no player");
        let Ok(dir) = Dir3::new((target.0 - player_transform.translation.to_2d()).to_3d(0.0))
        else {
            continue;
        };
        let ray = Ray3d::new(player_transform.translation, dir);

        let len = raycast
            .cast_ray(ray, &settings)
            .first()
            .map_or(asset_handler.target.laser_length, |hit| hit.1.distance);
        // We need to scale in the "y" direction because that's the orientation of
        // the cylinder that we use to draw the laser, it's just rotated.
        transform.scale.y = len;
        transform.translation.z = -len * 0.5;
    }
}

// Scenes have meshes as their descendents. There's probably a better way to do
// this.
fn scene_block_sight_system(
    mut commands: Commands,
    query: Query<Entity, (Added<SceneInstance>, Added<BlocksSight>)>,
    children: Query<&Children>,
    meshes: Query<&Mesh3d>,
) {
    for entity in &query {
        for descendant in children.iter_descendants(entity) {
            if meshes.get(descendant).is_ok() {
                commands.entity(descendant).insert(BlocksSight);
            }
        }
    }
}
