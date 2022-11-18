#![feature(once_cell)]

pub mod ability;
pub mod config;
pub mod healthbar;
pub mod system;

use bevy::{
    prelude::{
        default, shape, AssetServer, Assets, BuildChildren, Bundle, Camera, Camera3dBundle, Color,
        Commands, Component, ComputedVisibility, GlobalTransform, Handle, Mat4, Material, Mesh,
        OrthographicProjection, PbrBundle, PointLight, PointLightBundle, Quat, Query, Ray, Res,
        ResMut, StandardMaterial, Transform, Vec2, Vec3, Visibility,
    },
    render::camera::ScalingMode,
    scene::Scene,
    time::{Time, Timer},
    window::Windows,
};
use bevy_rapier3d::prelude::{Collider, LockedAxes, RigidBody, Velocity};
use rand::Rng;

use crate::{
    ability::{cooldown, HYPER_SPRINT_COOLDOWN, SHOOT_COOLDOWN},
    healthbar::{Healthbar, HealthbarMarker},
};

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

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCooldowns {
    hyper_sprint: Timer,
    shoot: Timer,
}

// TODO: Do cooldowns betterer
pub fn player_cooldown_system(mut player_cooldowns: Query<&mut PlayerCooldowns>, time: Res<Time>) {
    for mut pcd in player_cooldowns.iter_mut() {
        pcd.hyper_sprint.tick(time.delta());
        pcd.shoot.tick(time.delta());
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Ally;

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
const NUM_ENEMIES: u32 = 25;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let player = commands
        .spawn((
            Player,
            Character {
                health: Health::new(100.0),
                scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
                outline: meshes.add(
                    shape::Circle {
                        radius: 1.0,
                        vertices: 100,
                    }
                    .into(),
                ),
                material: materials.add(Color::CYAN.into()),
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
                visibility: Visibility::VISIBLE,
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(1.0),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed(SPEED),
                velocity: Velocity::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
            PlayerCooldowns {
                hyper_sprint: cooldown(HYPER_SPRINT_COOLDOWN),
                shoot: cooldown(SHOOT_COOLDOWN),
            },
        ))
        .id();
    let player_health_bar = commands
        .spawn(Healthbar {
            marker: HealthbarMarker,
            material: materials.add(Color::DARK_GREEN.into()),
            mesh: meshes.add(
                shape::Quad {
                    size: Vec2::new(1.8, 0.3),
                    ..default()
                }
                .into(),
            ),
            transform: Transform::from_xyz(0.0, -1.3, 0.01),
            global_transform: GlobalTransform::default(),
            visibility: Visibility::VISIBLE,
            computed_visibility: ComputedVisibility::default(),
        })
        .id();
    commands.entity(player).push_children(&[player_health_bar]);

    let mut rng = rand::thread_rng();
    for _ in 0..NUM_ENEMIES {
        let x = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        commands.spawn((
            Enemy,
            Character {
                health: Health::new(100.0),
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
                max_speed: MaxSpeed(SPEED),
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
