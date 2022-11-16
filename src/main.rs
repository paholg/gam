#![feature(once_cell)]

pub mod config;

use std::sync::LazyLock;

use bevy::{
    input::mouse::MouseMotion,
    prelude::{
        default, shape, App, AssetServer, Assets, Bundle, Camera, Camera3d, Camera3dBundle, Color,
        Commands, Component, EventReader, GlobalTransform, Image, Input, KeyCode, Mat4, Material,
        Mesh, Msaa, OrthographicProjection, PbrBundle, PointLightBundle, Query, Ray, Res, ResMut,
        StandardMaterial, Transform, Vec2, Vec3, With, Without,
    },
    render::camera::ScalingMode,
    sprite::ColorMaterial,
    window::Windows,
    DefaultPlugins,
};
use tracing::info;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Direction {
    phi: f32,
}

#[derive(Component)]
struct Health {
    cur: f32,
    max: f32,
}

#[derive(Component)]
struct Radius {
    r: f32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Bundle)]
struct CharacterBundle {
    pos: Position,
    dir: Direction,
    health: Health,
    radius: Radius,
}

const PLAYER_R: f32 = 1.0;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CharacterBundle {
            pos: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            dir: Direction { phi: 0.0 },
            health: Health {
                cur: 100.0,
                max: 100.0,
            },
            radius: Radius { r: PLAYER_R },
        },
        PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: 1.0,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::LIME_GREEN.into()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(PLAYER_R, PLAYER_R, PLAYER_R),
                ..default()
            },
            ..default()
        },
        Player,
    ));

    // ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(20.0, 20.0),
                ..default()
            }
            .into(),
        ),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

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
        transform: Transform::from_xyz(0.0, 0.0, 5.0),
        ..default()
    });
}

fn move_player(input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    const SPEED: f32 = 0.1;
    let mut transform = query.single_mut();
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
    transform.translation += delta;
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

/// Moves the camera and orients the player based on the mouse cursor.
fn move_camera(
    windows: Res<Windows>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let (mut transform, camera, global_transform) = camera_query.single_mut();
    let player_transform = player_query.single();

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
}

fn main() {
    App::new()
        .add_startup_system(setup)
        .add_system(move_player)
        .add_system(move_camera)
        .add_plugins(DefaultPlugins)
        .run();
}
