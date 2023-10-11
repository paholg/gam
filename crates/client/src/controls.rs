use std::f32::consts::PI;

use bevy::{
    prelude::{
        Camera, Commands, Entity, EventReader, GlobalTransform, NextState, Plugin, Quat, Query,
        Res, ResMut, Resource, State, Transform, Update, Vec3, With, Without,
    },
    window::{CursorMoved, PrimaryWindow, Window},
};
use bevy_rapier3d::prelude::{ExternalImpulse, RapierConfiguration, Velocity};
use leafwing_input_manager::prelude::ActionState;

use engine::{
    ability::{properties::AbilityProps, ABILITY_Z},
    pointing_angle,
    status_effect::{StatusEffect, StatusEffects},
    time::TickCounter,
    AppState, Cooldowns, Energy, FixedTimestepSystem, MaxSpeed, Player,
};

use crate::CAMERA_OFFSET;

use super::config::{Action, Config, ABILITY_COUNT};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (menu, player_aim, update_cursor))
            .add_engine_tick_systems((player_ability, player_movement))
            .insert_resource(CameraFollowMode::Mouse);
    }
}

#[derive(Resource, PartialEq, Eq, Debug, Clone, Copy)]
enum CameraFollowMode {
    Mouse,
    Controller,
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
        match state.get() {
            AppState::Loading => (),
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

fn player_ability(
    config: Res<Config>,
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    props: Res<AbilityProps>,
    mut query: Query<(
        Entity,
        &ActionState<Action>,
        &mut Energy,
        &mut Cooldowns,
        &mut Velocity,
        &mut StatusEffects,
        &Transform,
        &Player,
    )>,
) {
    let (
        entity,
        action_state,
        mut energy,
        mut cooldowns,
        velocity,
        mut status_effects,
        transform,
        player,
    ) = match query.get_single_mut() {
        Ok(q) => q,
        Err(_) => return,
    };

    // Abilities:
    for pressed in &action_state.get_pressed() {
        let just_pressed = action_state.just_pressed(*pressed);
        let pressed_usize = *pressed as usize;
        if pressed_usize < ABILITY_COUNT {
            let ability = &config.player.abilities[pressed_usize];
            ability.fire(
                just_pressed,
                &mut commands,
                &tick_counter,
                &props,
                entity,
                &mut energy,
                &mut cooldowns,
                transform,
                &velocity,
                &mut status_effects,
                player.target,
            );
        }
    }

    for released in &action_state.get_just_released() {
        let released_usize = *released as usize;
        if released_usize < ABILITY_COUNT {
            let ability = &config.player.abilities[released_usize];
            ability.unfire(&mut commands, entity, &mut status_effects);
        }
    }
}

// TODO: Some of this needs to go into engine.
fn player_movement(
    mut query: Query<
        (
            &ActionState<Action>,
            &mut ExternalImpulse,
            &MaxSpeed,
            &StatusEffects,
        ),
        With<Player>,
    >,
    props: Res<AbilityProps>,
) {
    let (action_state, mut impulse, max_speed, status_effects) =
        if let Ok(q) = query.get_single_mut() {
            q
        } else {
            return;
        };

    if action_state.pressed(Action::Move) {
        let axis_pair = action_state.clamped_axis_pair(Action::Move).unwrap();
        let dir = axis_pair.xy().extend(0.0);
        let mut max_impulse = max_speed.impulse;
        if status_effects
            .effects
            .contains(&StatusEffect::HyperSprinting)
        {
            max_impulse *= props.hyper_sprint.factor;
        }

        impulse.impulse = dir * max_impulse;
    }
}

const MAX_RANGE: f32 = 20.0;

fn player_aim(
    mut player_query: Query<(&ActionState<Action>, &mut Transform, &mut Player), Without<Camera>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    mut camera_mode: ResMut<CameraFollowMode>,
) {
    let (action_state, mut transform, mut player) = if let Ok(query) = player_query.get_single_mut()
    {
        query
    } else {
        return;
    };

    if action_state.pressed(Action::Aim) {
        let axis_pair = action_state.clamped_axis_pair(Action::Aim).unwrap();
        let rotation = axis_pair.rotation().unwrap().into_radians() - PI * 0.5;
        transform.rotation = Quat::from_axis_angle(Vec3::Z, rotation);

        player.target = transform.translation.truncate() + axis_pair.xy() * MAX_RANGE;

        *camera_mode = CameraFollowMode::Controller;
    }

    if *camera_mode == CameraFollowMode::Controller {
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
fn update_cursor(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<(&mut Transform, &mut Player), Without<Camera>>,
    cursor_events: EventReader<CursorMoved>,
    mut camera_mode: ResMut<CameraFollowMode>,
) {
    if cursor_events.is_empty() && *camera_mode == CameraFollowMode::Controller {
        return;
    }

    *camera_mode = CameraFollowMode::Mouse;

    let (mut camera_transform, camera, camera_global_transform) = camera_query.single_mut();
    let Ok((mut player_transform, mut player)) = player_query.get_single_mut() else {
        return;
    };

    let Some(cursor_window) = primary_window.single().cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_global_transform, cursor_window) else {
        return;
    };

    let Some(ground_distance) = ray.intersect_plane(Vec3::new(0.0, 0.0, 0.0), Vec3::Z) else {
        return;
    };
    player.target = ray.get_point(ground_distance).truncate();

    let Some(distance) = ray.intersect_plane(Vec3::new(0.0, 0.0, ABILITY_Z), Vec3::Z) else {
        return;
    };
    let cursor = ray.get_point(distance);

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