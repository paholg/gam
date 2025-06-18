use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::state::state::NextState;
use bevy::state::state::State;
use bevy::transform::components::Transform;

use crate::face;
use crate::movement::DesiredMove;
use crate::multiplayer::Action;
use crate::multiplayer::PlayerInputs;
use crate::player::Abilities;
use crate::AppState;
use crate::Player;
use crate::Target;

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
        if let Some(cursor) = input.cursor() {
            user.target.0 = cursor;
            face(&mut user.transform, cursor);
        }
        // Abilities
        let buttons = input.buttons();
        buttons.fire_abilities(&mut commands, user.entity, user.abilities);

        // Movement
        user.desired_move.dir = input.movement().clamp_length_max(1.0);
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
