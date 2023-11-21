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
    Player,
};

use crate::{config::UserAction, CAMERA_OFFSET};

pub struct ControlPlugin {
    pub player: Player,
}

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(CameraFollowMode::default())
            .insert_resource(self.player)
            .add_systems(Update, player_input);
    }
}

#[derive(Resource, PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum CameraFollowMode {
    Mouse,
    // Note: We set controller as default, as a simple mouse movement will
    // switch to mouse, but we require more work to set it to controller.
    #[default]
    Controller,
}

const MAX_RANGE: f32 = 20.0;

pub fn player_input(
    player: Res<Player>,
    mut player_inputs: ResMut<PlayerInputs>,
    player_query: Query<(&Player, &ActionState<UserAction>, &Transform), Without<Camera>>,
    primary_window: Query<&Window, (With<PrimaryWindow>, Without<Camera>)>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut Transform)>,
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
            UserAction::Menu => {
                // Only want to send the menu command when just pressed, or it
                // may flicker.
                if action_state.just_pressed(UserAction::Menu) {
                    action |= Action::Menu
                }
            }
            UserAction::Move | UserAction::Aim => (),
        }
    }

    let movement = action_state
        .clamped_axis_pair(UserAction::Move)
        .map(|pair| pair.xy())
        .unwrap_or_default();
    let (camera, camera_global_transform, mut camera_transform) = camera_query.single_mut();

    // Try to determine if the player wants to use mouse or controller to aim.
    let controller_aim = action_state
        .clamped_axis_pair(UserAction::Aim)
        .unwrap_or_default();
    if !cursor_events.is_empty() {
        *camera_mode = CameraFollowMode::Mouse;
    } else if controller_aim.length_squared() > 0.5 * 0.5 {
        *camera_mode = CameraFollowMode::Controller;
    }

    let cursor = match *camera_mode {
        CameraFollowMode::Mouse => {
            cursor_from_mouse(primary_window.single(), camera, camera_global_transform)
                .unwrap_or(player_transform.translation.truncate())
        }
        CameraFollowMode::Controller => {
            player_transform.translation.truncate() + controller_aim.xy() * MAX_RANGE
        }
    };

    let input = Input::new(action, movement, cursor);
    player_inputs.insert(player, input);

    // Update camera
    let camera_weight = 0.9;
    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at =
        cursor.extend(0.0) * CURSOR_WEIGHT + player_transform.translation * (1.0 - CURSOR_WEIGHT);
    let look_at = (camera_transform.translation - CAMERA_OFFSET) * camera_weight
        + look_at * (1.0 - camera_weight);

    *camera_transform =
        Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);
}

fn cursor_from_mouse(
    primary_window: &Window,
    camera: &Camera,
    camera_gt: &GlobalTransform,
) -> Option<Vec2> {
    let cursor_window = primary_window.cursor_position()?;

    let ray = camera.viewport_to_world(camera_gt, cursor_window)?;
    let distance = ray.intersect_plane(Vec3::new(0.0, 0.0, ABILITY_Z), Vec3::Z)?;
    let cursor = ray.get_point(distance);
    Some(cursor.truncate())
}
