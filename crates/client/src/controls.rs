use bevy::{
    prelude::{
        Camera, EventReader, GlobalTransform, Plugin, Query, Res, ResMut, Resource, Transform,
        Update, Vec2, Vec3, With, Without,
    },
    window::{CursorMoved, PrimaryWindow, Window},
};

use leafwing_input_manager::prelude::ActionState;

use engine::{
    ability::ABILITY_Z,
    multiplayer::{Action, Input, PlayerInputs},
    Player, Target,
};

use crate::{config::UserAction, CAMERA_OFFSET};

pub struct ControlPlugin {
    pub player: Player,
}

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(CameraFollowMode::Mouse)
            .insert_resource(self.player)
            .add_systems(Update, player_input)
            .add_systems(Update, update_cursor);
    }
}

#[derive(Resource, PartialEq, Eq, Debug, Clone, Copy)]
pub enum CameraFollowMode {
    Mouse,
    Controller,
}

const MAX_RANGE: f32 = 20.0;

pub fn player_input(
    player: Res<Player>,
    mut player_inputs: ResMut<PlayerInputs>,
    player_query: Query<(&Player, &ActionState<UserAction>, &Transform)>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut camera_mode: ResMut<CameraFollowMode>,
    cursor_events: EventReader<CursorMoved>,
) {
    let player = *player;
    let filtered_query = player_query.iter().find(|tuple| *tuple.0 == player);
    let Some((_, action_state, player_transform)) = filtered_query else {
        return;
    };

    let mut action = Action::none();
    for pressed in action_state.get_pressed() {
        match pressed {
            UserAction::Ability0 => action |= Action::Ability0,
            UserAction::Ability1 => action |= Action::Ability1,
            UserAction::Ability2 => action |= Action::Ability2,
            UserAction::Ability3 => action |= Action::Ability3,
            UserAction::Ability4 => action |= Action::Ability4,
            UserAction::Menu => action |= Action::Menu,
            UserAction::Move | UserAction::Aim => (),
        }
    }

    let movement = action_state
        .clamped_axis_pair(UserAction::Move)
        .map(|pair| pair.xy())
        .unwrap_or_default();

    let cursor = match action_state.clamped_axis_pair(UserAction::Aim) {
        Some(pair) => {
            *camera_mode = CameraFollowMode::Controller;
            player_transform.translation.truncate() + pair.xy() * MAX_RANGE
        }
        None => {
            if cursor_events.is_empty() && *camera_mode == CameraFollowMode::Controller {
                player_transform.translation.truncate()
            } else {
                *camera_mode = CameraFollowMode::Mouse;
                cursor_from_mouse(primary_window, camera_query)
                    .unwrap_or(player_transform.translation.truncate())
            }
        }
    };

    let input = Input::new(action, movement, cursor);
    player_inputs.insert(player, input);
}

fn cursor_from_mouse(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let cursor_window = primary_window.single().cursor_position()?;

    let (camera, camera_global_transform) = camera_query.get_single().ok()?;

    let ray = camera.viewport_to_world(camera_global_transform, cursor_window)?;
    let distance = ray.intersect_plane(Vec3::new(0.0, 0.0, ABILITY_Z), Vec3::Z)?;
    let cursor = ray.get_point(distance);
    Some(cursor.truncate())
}

/// Moves the camera based on the mouse cursor.
fn update_cursor(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<(&Transform, &mut Target), (With<Player>, Without<Camera>)>,
    cursor_events: EventReader<CursorMoved>,
    mut camera_mode: ResMut<CameraFollowMode>,
) {
    if cursor_events.is_empty() && *camera_mode == CameraFollowMode::Controller {
        return;
    }

    *camera_mode = CameraFollowMode::Mouse;

    let (mut camera_transform, camera, camera_global_transform) = camera_query.single_mut();
    let Ok((player_transform, mut target)) = player_query.get_single_mut() else {
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
    target.0 = ray.get_point(ground_distance).truncate();

    let Some(distance) = ray.intersect_plane(Vec3::new(0.0, 0.0, ABILITY_Z), Vec3::Z) else {
        return;
    };
    let cursor = ray.get_point(distance);

    let camera_weight = 0.9;
    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at = cursor * CURSOR_WEIGHT + player_transform.translation * (1.0 - CURSOR_WEIGHT);
    let look_at = (camera_transform.translation - CAMERA_OFFSET) * camera_weight
        + look_at * (1.0 - camera_weight);

    *camera_transform =
        Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);
}
