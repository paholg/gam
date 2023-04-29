use bevy::{
    prelude::{
        Camera, Commands, DespawnRecursiveExt, Entity, EventWriter, GlobalTransform, Input,
        KeyCode, MouseButton, NextState, Quat, Query, Res, ResMut, State, Transform, Vec2, Vec3,
        With, Without,
    },
    window::{PrimaryWindow, Window},
};
use bevy_rapier3d::prelude::{
    Collider, ExternalImpulse, LockedAxes, RapierConfiguration, ReadMassProperties, RigidBody,
    Velocity,
};
use rand::Rng;
use tracing::info;

use crate::{
    ability::{ABILITY_Z, HYPER_SPRINT_COOLDOWN, SHOOT_COOLDOWN, SHOTGUN_COOLDOWN},
    ai::simple::Attitude,
    config::Config,
    pointing_angle,
    time::TickCounter,
    Ai, Ally, AppState, Character, Cooldowns, DeathEvent, Enemy, Health, MaxSpeed, NumAi, Player,
    CAMERA_OFFSET, DAMPING, PLANE, PLAYER_R,
};

/// Player input that does not affect the game state, like menu toggle.
pub fn player_out_of_game_input(
    config: Res<Config>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut physics_config: ResMut<RapierConfiguration>,
) {
    let controls = &config.controls;

    if controls.menu.just_pressed(&keyboard_input, &mouse_input) {
        match state.0 {
            AppState::Running => {
                physics_config.physics_pipeline_active = false;
                next_state.set(AppState::Paused);
            }
            AppState::Paused => {
                physics_config.physics_pipeline_active = true;
                next_state.set(AppState::Running);
            }
        }
    }
}

pub fn player_input(
    config: Res<Config>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<
        (
            Entity,
            &mut Cooldowns,
            &mut Velocity,
            &mut MaxSpeed,
            &mut ExternalImpulse,
            &Transform,
        ),
        With<Player>,
    >,
) {
    let controls = &config.controls;

    let (entity, mut cooldowns, mut velocity, mut max_speed, mut impulse, transform) =
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
                &tick_counter,
                entity,
                &mut cooldowns,
                &mut max_speed,
                transform,
                &velocity,
            );
        }
    }

    // Movement:
    let mut new_impulse = Vec3::new(0.0, 0.0, 0.0);

    for (control, dir) in [
        (&controls.left, Vec3::new(-1.0, 0.0, 0.0)),
        (&controls.right, Vec3::new(1.0, 0.0, 0.0)),
        (&controls.up, Vec3::new(0.0, 1.0, 0.0)),
        (&controls.down, Vec3::new(0.0, -1.0, 0.0)),
    ] {
        if control.pressed(&keyboard_input, &mouse_input) {
            new_impulse += dir;
        }
    }

    new_impulse = new_impulse.clamp_length_max(1.0) * max_speed.impulse;

    impulse.impulse = new_impulse;
}

/// Moves the camera and orients the player based on the mouse cursor.
pub fn update_cursor(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
) {
    let (mut camera_transform, camera, camera_global_transform) = camera_query.single_mut();

    let Some(cursor_window) = primary_window.single().cursor_position() else { return; };

    let Some(ray) = camera.viewport_to_world(camera_global_transform, cursor_window) else { return; };

    let Some(distance) = ray.intersect_plane(Vec3::new(0.0, 0.0, ABILITY_Z), Vec3::Z) else { return; };
    let cursor = ray.get_point(distance);

    let Ok(mut player_transform) = player_query.get_single_mut() else { return; };
    let angle = pointing_angle(player_transform.translation, cursor);
    player_transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

    let camera_weight = 0.9;
    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at = cursor * CURSOR_WEIGHT + player_transform.translation * (1.0 - CURSOR_WEIGHT);
    let look_at = (camera_transform.translation - CAMERA_OFFSET) * camera_weight
        + look_at * (1.0 - camera_weight);

    *camera_transform =
        Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);
}

pub fn die(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform)>,
    mut event_writer: EventWriter<DeathEvent>,
) {
    for (entity, health, &transform) in query.iter() {
        if health.cur <= 0.0 {
            event_writer.send(DeathEvent { transform });
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Player,
        Ally,
        Character {
            health: Health::new(100.0),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            collider: Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0), 1.0),
            body: RigidBody::Dynamic,
            max_speed: MaxSpeed::default(),
            velocity: Velocity::default(),
            damping: DAMPING,
            impulse: ExternalImpulse::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            mass: ReadMassProperties::default(),
        },
        Cooldowns::default(),
    ));
}

pub fn point_in_plane() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen::<f32>() * (PLANE - PLAYER_R) - (PLANE - PLAYER_R) * 0.5;
    let y = rng.gen::<f32>() * (PLANE - PLAYER_R) - (PLANE - PLAYER_R) * 0.5;
    Vec3::new(x, y, 0.0)
}

fn spawn_enemies(commands: &mut Commands, num: usize) {
    for _ in 0..num {
        let loc = point_in_plane();
        commands.spawn((
            Enemy,
            Ai,
            Character {
                health: Health::new(10.0),
                transform: Transform::from_translation(loc),
                global_transform: GlobalTransform::default(),
                collider: Collider::capsule(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 2.0),
                    1.0,
                ),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed::default(),
                velocity: Velocity::default(),
                damping: DAMPING,
                impulse: ExternalImpulse::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                mass: ReadMassProperties::default(),
            },
            Cooldowns {
                hyper_sprint: HYPER_SPRINT_COOLDOWN,
                // FIXME: Figure out why this can't be zero.
                shoot: SHOOT_COOLDOWN,
                shotgun: SHOTGUN_COOLDOWN,
            },
            Attitude::rand(),
        ));
    }
}

fn spawn_allies(commands: &mut Commands, num: usize) {
    for _ in 0..num {
        let loc = point_in_plane();
        commands.spawn((
            Ally,
            Ai,
            Character {
                health: Health::new(100.0),
                transform: Transform::from_translation(loc),
                global_transform: GlobalTransform::default(),
                collider: Collider::capsule(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 2.0),
                    1.0,
                ),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed::default(),
                velocity: Velocity::default(),
                damping: DAMPING,
                impulse: ExternalImpulse::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                mass: ReadMassProperties::default(),
            },
            Cooldowns::default(),
        ));
    }
}

pub fn reset(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    ally_query: Query<Entity, With<Ally>>,
    player_query: Query<Entity, With<Player>>,
    mut num_ai: ResMut<NumAi>,
) {
    if enemy_query.iter().next().is_none() {
        num_ai.enemies += 1;
        spawn_enemies(&mut commands, num_ai.enemies);
    }

    #[cfg(not(feature = "train"))]
    {
        if player_query.iter().next().is_none() {
            num_ai.enemies = num_ai.enemies.saturating_sub(1);
            spawn_player(&mut commands);
        }
    }

    if ally_query.iter().next().is_none() {
        spawn_allies(&mut commands, num_ai.allies);
    }
}
