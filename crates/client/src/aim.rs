use std::f32::consts::PI;

use bevy::{
    app::Startup,
    asset::{Assets, Handle},
    color::LinearRgba,
    ecs::{
        hierarchy::{ChildOf, Children},
        resource::Resource,
        system::ResMut,
    },
    math::{
        primitives::{Cylinder, Sphere},
        Quat,
    },
    pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver, StandardMaterial},
    prelude::{
        Added, Commands, Component, Entity, Mesh3d, Plugin, Query, Res, Transform, Update, With,
        Without,
    },
    render::mesh::Mesh,
    scene::SceneInstance,
};
use engine::{Player, Target, UP};

use crate::in_plane;

/// A plugin for managing aiming, such as drawing and updating the cursor.
pub struct AimPlugin;

impl Plugin for AimPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, create_assets).add_systems(
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

#[derive(Resource)]
pub struct TargetAssets {
    pub cursor_mesh: Handle<Mesh>,
    pub cursor_material: Handle<StandardMaterial>,
    pub laser_mesh: Handle<Mesh>,
    pub laser_material: Handle<StandardMaterial>,
}

impl TargetAssets {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let target_material = StandardMaterial {
            emissive: LinearRgba::rgb(10.0, 0.0, 0.1),
            ..Default::default()
        };

        let target_laser_material = StandardMaterial {
            emissive: LinearRgba::rgb(10.0, 0.0, 0.1),
            ..Default::default()
        };
        let laser_mesh =
            Mesh::from(Cylinder::new(0.01, 1.0)).rotated_by(Quat::from_rotation_x(PI / 2.0));
        TargetAssets {
            cursor_mesh: meshes.add(Sphere::new(0.06)),
            cursor_material: materials.add(target_material),
            laser_mesh: meshes.add(laser_mesh),
            laser_material: materials.add(target_laser_material),
        }
    }
}

fn create_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(TargetAssets::new(&mut meshes, &mut materials));
}

#[derive(Component)]
struct CursorTarget;

fn draw_target_system(
    mut commands: Commands,
    query: Query<(Entity, &Target), Added<Player>>,
    assets: Res<TargetAssets>,
) {
    for (entity, target) in &query {
        let target_entity = commands
            .spawn((
                Mesh3d(assets.cursor_mesh.clone_weak()),
                MeshMaterial3d(assets.cursor_material.clone_weak()),
                Transform::from_translation(target.transform.translation),
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
    mut target_query: Query<(&ChildOf, &mut Transform), (Without<Player>, With<CursorTarget>)>,
) {
    for (child_of, mut transform) in &mut target_query {
        if let Ok((player_transform, target)) = player_query.get(child_of.parent()) {
            let mut t = in_plane();
            let rotation = player_transform.rotation.inverse();
            t.rotate(rotation);
            t.translation =
                rotation * (target.transform.translation - player_transform.translation);

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
    query: Query<Entity, Added<Player>>,
    assets: Res<TargetAssets>,
) {
    for entity in &query {
        let laser = commands
            .spawn((
                Mesh3d(assets.laser_mesh.clone_weak()),
                MeshMaterial3d(assets.laser_material.clone_weak()),
                Transform::default(),
                NotShadowCaster,
                NotShadowReceiver,
                LaserSight {},
            ))
            .id();

        commands.entity(entity).add_children(&[laser]);
    }
}

fn update_laser_system(
    mut laser_query: Query<
        (&ChildOf, &mut Transform),
        (With<LaserSight>, Without<Player>, Without<BlocksSight>),
    >,
    player_query: Query<(&Transform, &Target), With<Player>>,
) {
    for (child_of, mut transform) in &mut laser_query {
        let (player_transform, target) = player_query.get(child_of.parent()).expect("no player");

        let len = (target.transform.translation - player_transform.translation).length();
        transform.rotation = player_transform.rotation.inverse()
            * player_transform
                .looking_at(target.transform.translation, UP)
                .rotation;
        transform.translation = 0.5 * len * transform.forward();
        transform.scale.z = len;
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
