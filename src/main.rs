#![feature(once_cell)]

pub mod config;

use std::{f32::consts::PI, sync::LazyLock};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::mouse::MouseMotion,
    prelude::{
        default, shape, AmbientLight, App, AssetServer, Assets, Bundle, Camera, Camera3d,
        Camera3dBundle, Color, Commands, Component, ComputedVisibility, DirectionalLight,
        DirectionalLightBundle, EventReader, GlobalTransform, Handle, Image, Input, KeyCode, Mat4,
        Material, Mesh, Msaa, OrthographicProjection, PbrBundle, PointLight, PointLightBundle,
        Quat, Query, Ray, Res, ResMut, StandardMaterial, Transform, Vec2, Vec3, Visibility, With,
        Without,
    },
    render::camera::ScalingMode,
    scene::{Scene, SceneBundle},
    sprite::ColorMaterial,
    window::Windows,
    DefaultPlugins,
};
use bevy_rapier3d::{
    prelude::{
        Collider, Friction, LockedAxes, NoUserData, RapierConfiguration, RapierPhysicsPlugin,
        RigidBody, Velocity,
    },
    rapier::prelude::ColliderBuilder,
    render::RapierDebugRenderPlugin,
};
use rand::Rng;
use tracing::info;

#[derive(Component)]
struct Health {
    cur: f32,
    max: f32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Bundle)]
struct CharacterBundle<M: Material> {
    health: Health,
    scene: Handle<Scene>,
    outline: Handle<Mesh>,
    material: Handle<M>,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    collider: Collider,
    body: RigidBody,
    velocity: Velocity,
    locked_axes: LockedAxes,
}

const PLAYER_R: f32 = 1.0;
const SPEED: f32 = 10.0;
const NUM_ENEMIES: u32 = 25;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Player,
        CharacterBundle {
            health: Health {
                cur: 100.0,
                max: 100.0,
            },
            scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
            outline: meshes.add(
                shape::Circle {
                    radius: 1.0,
                    vertices: 100,
                }
                .into(),
            ),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::VISIBLE,
            computed_visibility: ComputedVisibility::default(),
            collider: Collider::ball(1.0),
            body: RigidBody::Dynamic,
            velocity: Velocity::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
        },
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..NUM_ENEMIES {
        let x = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        commands.spawn((
            Enemy,
            CharacterBundle {
                health: Health {
                    cur: 100.0,
                    max: 100.0,
                },
                scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
                outline: meshes.add(
                    shape::Circle {
                        radius: 1.0,
                        vertices: 100,
                    }
                    .into(),
                ),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_xyz(x, y, 0.0),
                global_transform: GlobalTransform::default(),
                visibility: Visibility::VISIBLE,
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(1.0),
                body: RigidBody::Dynamic,
                velocity: Velocity::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
        ));
    }

    // ground plane
    const PLANE_SIZE: f32 = 30.0;
    let collider = Collider::compound(vec![
        (
            Vec3::new(PLANE_SIZE * 1.5, 0.0, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE, PLANE_SIZE),
        ),
        (
            Vec3::new(-PLANE_SIZE * 1.5, 0.0, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE, PLANE_SIZE),
        ),
        (
            Vec3::new(0.0, PLANE_SIZE * 1.5, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE, PLANE_SIZE),
        ),
        (
            Vec3::new(0.0, -PLANE_SIZE * 1.5, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE, PLANE_SIZE),
        ),
    ]);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                shape::Quad {
                    size: Vec2::new(PLANE_SIZE, PLANE_SIZE),
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::SILVER.into()),
            transform: Transform::from_xyz(0.0, 0.0, -0.1),
            ..default()
        },
        RigidBody::KinematicPositionBased,
        collider,
    ));

    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 10.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE_SIZE,
            intensity: 2000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 5.0),
        ..default()
    });
}

fn move_player(input: Res<Input<KeyCode>>, mut query: Query<&mut Velocity, With<Player>>) {
    let mut velocity = query.single_mut();
    let mut vec = Vec2::new(0.0, 0.0);

    let controls = &config::config().controls;

    if input.pressed(controls.left) {
        vec += Vec2::new(-SPEED, 0.0);
    }
    if input.pressed(controls.right) {
        vec += Vec2::new(SPEED, 0.0);
    }
    if input.pressed(controls.up) {
        vec += Vec2::new(0.0, SPEED);
    }
    if input.pressed(controls.down) {
        vec += Vec2::new(0.0, -SPEED);
    }
    vec = vec.clamp_length_max(SPEED);

    let delta = Vec3::new(vec.x, vec.y, 0.0);
    velocity.linvel = delta;
}

// Logic taken from here:
// https://github.com/lucaspoffo/renet/blob/c963b65b66325c536d115faab31638f3ad2b5e48/demo_bevy/src/lib.rs#L196-L215
fn ray_from_screenspace(
    windows: &Res<Windows>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray> {
    let window = windows.get_primary().unwrap();
    let cursor_position = window.cursor_position()?;

    let view = camera_transform.compute_matrix();
    let screen_size = camera.logical_target_size()?;
    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (cursor_position / screen_size) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;

    Some(Ray {
        origin: near,
        direction: ray_direction,
    })
}

// Logic taken from here:
// https://github.com/lucaspoffo/renet/blob/c963b65b66325c536d115faab31638f3ad2b5e48/demo_bevy/src/lib.rs#L217-L228
fn intersect_xy_plane(ray: &Ray, z_offset: f32) -> Option<Vec3> {
    let plane_normal = Vec3::Z;
    let plane_origin = Vec3::new(0.0, z_offset, 0.0);
    let denominator = ray.direction.dot(plane_normal);
    if denominator.abs() > f32::EPSILON {
        let point_to_point = plane_origin * z_offset - ray.origin;
        let intersect_dist = plane_normal.dot(point_to_point) / denominator;
        let intersect_position = ray.direction * intersect_dist + ray.origin;
        Some(intersect_position)
    } else {
        None
    }
}

// Returns an angle of rotation, along the z-axis, so that `from` will be pointing to `to`
fn pointing_angle(from: Vec3, to: Vec3) -> f32 {
    let dir = to - from;
    let angle = dir.angle_between(Vec3::Y);
    if dir.x > 0.0 {
        angle * -1.0
    } else {
        angle
    }
}

/// Moves the camera and orients the player based on the mouse cursor.
fn update_cursor(
    windows: Res<Windows>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
) {
    let (mut transform, camera, global_transform) = camera_query.single_mut();
    let mut player_transform = player_query.single_mut();

    let cursor = match ray_from_screenspace(&windows, camera, global_transform)
        .as_ref()
        .and_then(|ray| intersect_xy_plane(ray, 0.0))
    {
        Some(ray) => ray,
        None => return,
    };

    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at = cursor * CURSOR_WEIGHT + player_transform.translation * (1.0 - CURSOR_WEIGHT);

    *transform = Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);

    let angle = pointing_angle(player_transform.translation, cursor);
    player_transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
}

fn update_enemy_orientation(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut Transform, &mut Velocity), (With<Enemy>, Without<Player>)>,
) {
    let player_transform = player_query.single();
    enemy_query.for_each_mut(|(mut transform, mut velocity)| {
        let angle = pointing_angle(transform.translation, player_transform.translation);
        if !angle.is_nan() {
            transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
        }
        // Stop sliding after collisions
        velocity.linvel *= 0.9;
    });
}

fn main() {
    let mut rapier_config = RapierConfiguration::default();
    rapier_config.gravity = Vec3::ZERO;
    App::new()
        .add_startup_system(setup)
        .add_system(move_player)
        .add_system(update_cursor)
        .add_system(update_enemy_orientation)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(rapier_config)
        // .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
