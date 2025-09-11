use bevy::{
    ecs::{
        entity::Entity,
        query::QueryData,
        system::{Commands, Query, Res, ResMut},
    },
    state::state::{NextState, State},
    transform::components::Transform,
};

use crate::{
    movement::DesiredMove,
    multiplayer::{Action, PlayerInputs},
    player::Abilities,
    AppState, Player, Target,
};

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
    mut query: Query<InputUser>,
) {
    for mut user in query.iter_mut() {
        let Some(input) = inputs.get(user.player) else {
            continue;
        };

        // Targeting
        // FIXME
        // if let Some(cursor) = input.cursor() {
        //     user.target.0 = cursor;
        //     face(&mut user.transform, cursor);
        // }
        // Abilities
        let buttons = input.buttons();
        buttons.fire_abilities(&mut commands, user.entity, user.abilities);

        // Movement
        user.desired_move.vec = input.movement().clamp_length_max(1.0);
    }
}

pub fn pause_resume(
    inputs: Res<PlayerInputs>,
    query: Query<&Player>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for player in query.iter() {
        let Some(input) = inputs.get(player) else {
            continue;
        };

        if input.buttons().contains(Action::Menu) {
            match state.get() {
                AppState::Loading => {}
                AppState::Running => {
                    next_state.set(AppState::Menu);
                }
                AppState::Menu => {
                    next_state.set(AppState::Running);
                }
            }
        }
    }
}
