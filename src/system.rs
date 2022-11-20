use bevy::{
    prelude::{
        shape, AssetServer, Assets, Camera, Color, Commands, ComputedVisibility,
        DespawnRecursiveExt, Entity, GlobalTransform, Input, KeyCode, Mesh, MouseButton, Quat,
        Query, Res, ResMut, StandardMaterial, Transform, Vec2, Vec3, Visibility, With, Without,
    },
    window::Windows,
};
use bevy_rapier3d::prelude::{Collider, LockedAxes, RigidBody, Velocity};
use rand::Rng;

use crate::{
    ability::{cooldown, HYPER_SPRINT_COOLDOWN, SHOOT_COOLDOWN},
    config::config,
    healthbar::Healthbar,
    intersect_xy_plane, pointing_angle, ray_from_screenspace, Ai, Ally, Character, Cooldowns,
    Enemy, Health, MaxSpeed, NumAi, Player, CAMERA_OFFSET, PLANE_SIZE, PLAYER_R, SPEED,
};

pub fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Cooldowns,
            &mut Velocity,
            &mut MaxSpeed,
            &Transform,
        ),
        With<Player>,
    >,
) {
    let config = config();
    let controls = &config.controls;

    let (entity, mut cooldowns, mut velocity, mut max_speed, transform) =
        match query.get_single_mut() {
            Ok(q) => q,
            Err(_) => return,
        };

    // Abilities:
    let abilities = &config.player.abilities;
    for (control, ability) in controls.abilities.iter().zip(abilities.iter()) {
        if control.pressed(&keyboard_input, &mouse_input) {
            ability.fire(
                &mut commands,
                &mut meshes,
                &mut materials,
                entity,
                &mut cooldowns,
                &mut max_speed,
                &transform,
                &velocity,
            );
        }
    }

    // Movement:
    let mut delta_v = Vec2::new(0.0, 0.0);

    for (control, dir) in [
        (&controls.left, Vec2::new(-1.0, 0.0)),
        (&controls.right, Vec2::new(1.0, 0.0)),
        (&controls.up, Vec2::new(0.0, 1.0)),
        (&controls.down, Vec2::new(0.0, -1.0)),
    ] {
        if control.pressed(&keyboard_input, &mouse_input) {
            delta_v += dir;
        }
    }

    delta_v = delta_v.clamp_length_max(1.0) * max_speed.0;

    velocity.linvel = delta_v.extend(0.0);
}

/// Moves the camera and orients the player based on the mouse cursor.
pub fn update_cursor(
    windows: Res<Windows>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
) {
    let (mut transform, camera, global_transform) = camera_query.single_mut();

    let cursor = match ray_from_screenspace(&windows, camera, global_transform)
        .as_ref()
        .and_then(|ray| intersect_xy_plane(ray, 0.0))
    {
        Some(ray) => ray,
        None => return,
    };

    let player_translation = match player_query.get_single_mut() {
        Ok(mut player_transform) => {
            let angle = pointing_angle(player_transform.translation, cursor);
            player_transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

            player_transform.translation
        }
        Err(_) => {
            // No player; let's just keep things mostly centered for now.
            Vec3::default()
        }
    };

    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at = cursor * CURSOR_WEIGHT + player_translation * (1.0 - CURSOR_WEIGHT);

    *transform = Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);
}

pub fn die(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.cur <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &mut Res<AssetServer>,
) {
    commands.spawn((
        Player,
        Ally,
        Character {
            health: Health::new(100.0),
            healthbar: Healthbar::default(),
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
            max_speed: MaxSpeed(SPEED),
            velocity: Velocity::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
        },
        Cooldowns {
            hyper_sprint: cooldown(HYPER_SPRINT_COOLDOWN),
            shoot: cooldown(SHOOT_COOLDOWN),
        },
    ));
}

fn spawn_enemies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &mut Res<AssetServer>,
    num: u32,
) {
    let mut rng = rand::thread_rng();
    for _ in 0..num {
        let x = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        commands.spawn((
            Enemy,
            Ai,
            Character {
                health: Health::new(100.0),
                healthbar: Healthbar::default(),
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
            // TODO: Refactor cooldowns. This is temporary.
            Cooldowns {
                hyper_sprint: cooldown(HYPER_SPRINT_COOLDOWN),
                shoot: cooldown(SHOOT_COOLDOWN * 10),
            },
        ));
    }
}

fn spawn_allies(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &mut Res<AssetServer>,
    num: u32,
) {
    let mut rng = rand::thread_rng();
    for _ in 0..num {
        let x = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        commands.spawn((
            Ally,
            Ai,
            Character {
                health: Health::new(100.0),
                healthbar: Healthbar::default(),
                scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
                outline: meshes.add(
                    shape::Circle {
                        radius: 1.0,
                        vertices: 100,
                    }
                    .into(),
                ),
                material: materials.add(Color::CYAN.into()),
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
            // TODO: Refactor cooldowns. This is temporary.
            Cooldowns {
                hyper_sprint: cooldown(HYPER_SPRINT_COOLDOWN),
                shoot: cooldown(SHOOT_COOLDOWN * 10),
            },
        ));
    }
}

pub fn reset(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: Res<AssetServer>,
    mut num_ai: ResMut<NumAi>,
    enemy_query: Query<Entity, With<Enemy>>,
    ally_query: Query<Entity, With<Ally>>,
) {
    if enemy_query.iter().next().is_none() {
        num_ai.enemies += 1;
        spawn_enemies(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut asset_server,
            num_ai.enemies,
        );
    }

    if ally_query.iter().next().is_none() {
        num_ai.allies += 1;
        spawn_player(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut asset_server,
        );
        spawn_allies(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut asset_server,
            num_ai.allies,
        );
    }
}
