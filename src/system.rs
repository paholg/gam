use bevy::{
    prelude::{
        Assets, Camera, Commands, Entity, GlobalTransform, Input, KeyCode, Mesh, MouseButton, Quat,
        Query, Res, ResMut, StandardMaterial, Transform, Vec2, Vec3, With, Without,
    },
    window::Windows,
};
use bevy_rapier3d::prelude::Velocity;

use crate::{
    config::config, intersect_xy_plane, pointing_angle, ray_from_screenspace, Enemy, Health,
    MaxSpeed, Player, PlayerCooldowns, CAMERA_OFFSET,
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
            &mut PlayerCooldowns,
            &mut Velocity,
            &mut MaxSpeed,
            &Transform,
        ),
        With<Player>,
    >,
) {
    let config = config();
    let controls = &config.controls;

    let (entity, mut player_cooldowns, mut velocity, mut max_speed, transform) = query.single_mut();

    // Abilities:
    let abilities = &config.player.abilities;
    for (control, ability) in controls.abilities.iter().zip(abilities.iter()) {
        if control.just_pressed(&keyboard_input, &mouse_input) {
            ability.fire(
                &mut commands,
                &mut meshes,
                &mut materials,
                entity,
                &mut player_cooldowns,
                &mut max_speed,
                &transform,
                &velocity,
            )
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

// TODO: This should be part of whatever AI we're using.
pub fn update_enemy_orientation(
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

pub fn die(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.cur <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
