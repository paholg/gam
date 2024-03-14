use bevy_ecs::{
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

pub fn apply_inputs(
    inputs: Res<PlayerInputs>,
    mut commands: Commands,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<(
        &Abilities,
        &Player,
        &mut Transform,
        &mut Target,
        &mut DesiredMove,
    )>,
) {
    for (abilities, player, mut transform, mut target, mut desired_move) in query.iter_mut() {
        let Some(input) = inputs.get(player) else {
            continue;
        };

        // Targeting
        if let Some(cursor) = input.cursor() {
            target.0 = cursor;
            face(&mut transform, cursor);
        }

        // Abilities
        let buttons = input.buttons();
        for ability in buttons.abilities_fired(abilities) {
            commands.add(ability);
        }

        // Movement
        desired_move.dir = input.movement().clamp_length_max(1.0);

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
