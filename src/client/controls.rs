use std::f32::consts::PI;

use bevy::{
    prelude::{
        Camera, Commands, Entity, EventReader, GlobalTransform, NextState, Plugin, Quat, Query,
        Res, ResMut, State, Transform, Vec3, With, Without,
    },
    window::{CursorMoved, PrimaryWindow, Window},
};
use bevy_rapier3d::prelude::{ExternalImpulse, RapierConfiguration, Velocity};
use leafwing_input_manager::prelude::ActionState;
use tracing::info;

use crate::{
    ability::ABILITY_Z, pointing_angle, time::TickCounter, AppState, Cooldowns,
    FixedTimestepSystem, MaxSpeed, Player, CAMERA_OFFSET,
};

use super::config::{Action, Config, ABILITY_COUNT};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(menu)
            .add_engine_tick_system(player_ability)
            .add_engine_tick_system(player_movement)
            .add_system(player_aim)
            .add_system(update_cursor);
    }
}

fn menu(
    query: Query<&ActionState<Action>, With<Player>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut physics_config: ResMut<RapierConfiguration>,
) {
    let action_state = if let Ok(action_state) = query.get_single() {
        action_state
    } else {
        return;
    };

    if action_state.just_pressed(Action::Menu) {
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

pub fn player_ability(
    config: Res<Config>,
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<
        (
            Entity,
            &ActionState<Action>,
            &mut Cooldowns,
            &mut Velocity,
            &mut MaxSpeed,
            &Transform,
        ),
        With<Player>,
    >,
) {
    let (entity, action_state, mut cooldowns, velocity, mut max_speed, transform) =
        match query.get_single_mut() {
            Ok(q) => q,
            Err(_) => return,
        };

    // Abilities:
    for pressed in &action_state.get_pressed() {
        let pressed_usize = *pressed as usize;
        if pressed_usize < ABILITY_COUNT {
            let ability = &config.player.abilities[pressed_usize];
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
}

pub fn player_movement(
    mut query: Query<(&ActionState<Action>, &mut ExternalImpulse, &MaxSpeed), With<Player>>,
) {
    let (action_state, mut impulse, max_speed) = if let Ok(q) = query.get_single_mut() {
        q
    } else {
        return;
    };

    if action_state.pressed(Action::Move) {
        let axis_pair = action_state.clamped_axis_pair(Action::Move).unwrap();
        let dir = axis_pair.xy().extend(0.0);
        impulse.impulse = dir * max_speed.impulse;
    }
}

pub fn player_aim(
    mut player_query: Query<
        (&ActionState<Action>, &mut Transform),
        (With<Player>, Without<Camera>),
    >,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let (action_state, mut transform) = if let Ok(query) = player_query.get_single_mut() {
        query
    } else {
        return;
    };

    if action_state.pressed(Action::Aim) {
        let axis_pair = action_state.clamped_axis_pair(Action::Aim).unwrap();
        info!(?axis_pair);
        let rotation = axis_pair.rotation().unwrap().into_radians() - PI * 0.5;
        transform.rotation = Quat::from_axis_angle(Vec3::Z, rotation);

        // TODO: The camera should just follow the player until they use the mouse again.
        let mut camera_transform = if let Ok(query) = camera_query.get_single_mut() {
            query
        } else {
            return;
        };
        let look_at = transform.translation;
        *camera_transform =
            Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);
    }
}

/// Moves the camera and orients the player based on the mouse cursor.
pub fn update_cursor(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
    cursor_events: EventReader<CursorMoved>,
) {
    // We want to be sure to only run this logic if the cursor moves; otherwise
    // we can mess with rotation for controller users.
    if cursor_events.is_empty() {
        return;
    }

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
