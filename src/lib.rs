#![feature(once_cell)]

pub mod ability;
pub mod ai;
pub mod config;
pub mod healthbar;
pub mod system;

use bevy::{
    prelude::{
        default, shape, Assets, Bundle, Camera, Camera3dBundle, Color, Commands, Component,
        ComputedVisibility, GlobalTransform, Handle, Mat4, Material, Mesh, OrthographicProjection,
        PbrBundle, PointLight, PointLightBundle, Quat, Query, Ray, Res, ResMut, Resource,
        StandardMaterial, Transform, Vec2, Vec3, Visibility,
    },
    render::camera::ScalingMode,
    scene::Scene,
    time::{Time, Timer},
    window::Windows,
};
use bevy_rapier3d::prelude::{Collider, LockedAxes, RigidBody, Velocity};

use crate::healthbar::Healthbar;

#[derive(Component)]
pub struct Health {
    cur: f32,
    max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { cur: max, max }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct MaxSpeed(f32);

/// Indicate this entity is a player. Currently, we assume one player.
#[derive(Component)]
pub struct Player;

/// Indicate this entity is controlled by AI.
#[derive(Component)]
pub struct Ai;

/// Indicate this entity is on the enemy team.
#[derive(Component)]
pub struct Enemy;

/// Indicate this entity is on the players' team.
#[derive(Component)]
pub struct Ally;

#[derive(Component)]
pub struct Cooldowns {
    hyper_sprint: Timer,
    shoot: Timer,
}

// TODO: Do cooldowns betterer
pub fn player_cooldown_system(mut player_cooldowns: Query<&mut Cooldowns>, time: Res<Time>) {
    for mut pcd in player_cooldowns.iter_mut() {
        pcd.hyper_sprint.tick(time.delta());
        pcd.shoot.tick(time.delta());
    }
}

#[derive(Bundle)]
pub struct Object<M: Material> {
    material: Handle<M>,
    mesh: Handle<Mesh>,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    collider: Collider,
    body: RigidBody,
    velocity: Velocity,
    locked_axes: LockedAxes,
}

#[derive(Bundle)]
struct Character<M: Material> {
    health: Health,
    healthbar: Healthbar,
    scene: Handle<Scene>,
    outline: Handle<Mesh>,
    material: Handle<M>,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    collider: Collider,
    body: RigidBody,
    max_speed: MaxSpeed,
    velocity: Velocity,
    locked_axes: LockedAxes,
}

const PLAYER_R: f32 = 1.0;
const SPEED: f32 = 15.0;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

pub const PLANE_SIZE: f32 = 30.0;

#[derive(Resource)]
pub struct NumAi {
    pub enemies: u32,
    pub allies: u32,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
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

    // Camera
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

    // Light
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
