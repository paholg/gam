use bevy_ecs::{
    entity::Entity,
    query::QueryData,
    schedule::{NextState, State},
    system::{Commands, Query, Res, ResMut},
};
use bevy_transform::components::Transform;

use crate::{
    face,
    movement::DesiredMove,
    multiplayer::{Action, PlayerInputs},
    player::Abilities,
    AppState, Player, Target,
};

pub fn check_resume(
    inputs: Res<PlayerInputs>,
    query: Query<&Player>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for player in query.iter() {
        let Some(input) = inputs.get(player) else {
            continue;
        };
        let buttons = input.buttons();

        if buttons.contains(Action::Pause) {
            pause_resume(&state, &mut next_state);
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct InputUser {
    entity: Entity,
    abilities: &'static Abilities,
    player: &'static Player,
    transform: &'static mut Transform,
    target: &'static mut Target,
    desired_move: &'static mut DesiredMove,
}

pub fn apply_inputs(
    inputs: Res<PlayerInputs>,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<InputUser>,
) {
    for mut user in query.iter_mut() {
        let Some(input) = inputs.get(user.player) else {
            continue;
        };

        // Targeting
        if let Some(cursor) = input.cursor() {
            user.target.0 = cursor;
            face(&mut user.transform, cursor);
        }

        // Abilities
        let buttons = input.buttons();
        buttons.fire_abilities(&mut commands, user.entity, user.abilities);

        // Movement
        user.desired_move.dir = input.movement().clamp_length_max(1.0);

        // Menu
        if buttons.contains(Action::Pause) {
            pause_resume(&state, &mut next_state);
        }
    }
}

fn pause_resume(state: &State<AppState>, next_state: &mut NextState<AppState>) {
    match state.get() {
        AppState::Loading => {}
        AppState::Running => {
            next_state.set(AppState::Paused);
        }
        AppState::Paused => {
            next_state.set(AppState::Running);
        }
    }
}
