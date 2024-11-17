use bevy::prelude::Camera;
use bevy::prelude::EventReader;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::Transform;
use bevy::prelude::Update;
use bevy::prelude::Vec2;
use bevy::prelude::With;
use bevy::prelude::Without;
use bevy::time::Time;
use bevy::window::CursorMoved;
use bevy::window::PrimaryWindow;
use bevy::window::Window;
use engine::multiplayer::Action;
use engine::multiplayer::Input;
use engine::multiplayer::PlayerInputs;
use engine::Player;
use engine::To2d;
use engine::To3d;
use engine::ABILITY_Y;
use engine::UP;
use engine::UP_PLANE;
use leafwing_input_manager::prelude::ActionState;

use crate::config::GameAction;
use crate::config::UserAction;
use crate::CAMERA_OFFSET;

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

const MAX_RANGE: f32 = 7.0;

pub fn player_input(
    player: Res<Player>,
    mut player_inputs: ResMut<PlayerInputs>,
    player_query: Query<(&Player, &ActionState<UserAction>, &Transform), Without<Camera>>,
    primary_window: Query<&Window, (With<PrimaryWindow>, Without<Camera>)>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut Transform)>,
    mut camera_mode: ResMut<CameraFollowMode>,
    cursor_events: EventReader<CursorMoved>,
    time: Res<Time>,
) {
    let player = *player;
    let filtered_query = player_query.iter().find(|tuple| *tuple.0 == player);
    let Some((_, action_state, player_transform)) = filtered_query else {
        return;
    };

    let mut actions = action_state
        .get_pressed()
        .into_iter()
        .filter_map(|a| GameAction::try_from(a).ok())
        .map(Action::from)
        .fold(Action::none(), |acc, a| acc | a);

    // Handle menu separately, as we only want to send it when `just_pressed`
    // to prevent flickering.
    if action_state.just_pressed(&UserAction::Menu) {
        actions |= Action::Pause;
    }

    let movement = action_state.clamped_axis_pair(&UserAction::Move);
    let (camera, camera_global_transform, mut camera_transform) = camera_query.single_mut();

    // Try to determine if the player wants to use mouse or controller to aim.
    let controller_aim = action_state.clamped_axis_pair(&UserAction::Aim);
    if !cursor_events.is_empty() {
        *camera_mode = CameraFollowMode::Mouse;
    } else if controller_aim.length_squared() > 0.5 * 0.5 {
        *camera_mode = CameraFollowMode::Controller;
    }

    let cursor = match *camera_mode {
        CameraFollowMode::Mouse => {
            cursor_from_mouse(primary_window.single(), camera, camera_global_transform)
                .unwrap_or(player_transform.translation.to_2d())
        }
        CameraFollowMode::Controller => {
            player_transform.translation.to_2d() + controller_aim * MAX_RANGE
        }
    };

    let input = Input::new(actions, movement, cursor);
    player_inputs.insert(player, input);

    // Update camera
    const CAMERA_SPEED: f32 = 10.0;
    let camera_weight = 0.9;
    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at =
        cursor.to_3d(0.0) * CURSOR_WEIGHT + player_transform.translation * (1.0 - CURSOR_WEIGHT);
    let look_at = (camera_transform.translation - CAMERA_OFFSET) * camera_weight
        + look_at * (1.0 - camera_weight);

    let desired_transform =
        Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, UP);

    let delta = desired_transform.translation - camera_transform.translation;
    let max = delta.length();

    camera_transform.translation +=
        (delta.normalize_or_zero() * CAMERA_SPEED * time.delta_seconds()).clamp_length_max(max);
}

fn cursor_from_mouse(
    primary_window: &Window,
    camera: &Camera,
    camera_gt: &GlobalTransform,
) -> Option<Vec2> {
    let cursor_window = primary_window.cursor_position()?;

    let ray = camera.viewport_to_world(camera_gt, cursor_window)?;
    let distance = ray.intersect_plane(ABILITY_Y, UP_PLANE)?;
    let cursor = ray.get_point(distance);
    Some(cursor.to_2d())
}
