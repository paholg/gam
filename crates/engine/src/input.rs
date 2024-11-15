use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use bevy_state::state::NextState;
use bevy_state::state::State;
use bevy_transform::components::Transform;

use crate::face;
use crate::movement::DesiredMove;
use crate::multiplayer::Action;
use crate::multiplayer::PlayerInputs;
use crate::player::Abilities;
use crate::AppState;
use crate::Player;
use crate::Target;

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

// pub fn apply_inputs(
//     inputs: Res<PlayerInputs>,
//     mut commands: Commands,
//     props: Res<AbilityProps>,
//     state: Res<State<AppState>>,
//     mut next_state: ResMut<NextState<AppState>>,
//     mut query: Query<(
//         Entity,
//         &mut Energy,
//         &mut Cooldowns,
//         &Velocity,
//         &mut Transform,
//         &Player,
//         &mut Target,
//         &Abilities,
//         &mut DesiredMove,
//         &AbilityOffset,
//         &FootOffset,
//         &mut TimeDilation,
//     )>,
// ) {
//     for (
//         entity,
//         mut energy,
//         mut cooldowns,
//         velocity,
//         mut transform,
//         player,
//         mut target,
//         abilities,
//         mut desired_move,
//         ability_offset,
//         foot_offset,
//         mut time_dilation,
//     ) in query.iter_mut()
//     {
//         let Some(input) = inputs.get(player) else {
//             continue;
//         };

//         // Targeting
//         if let Some(cursor) = input.cursor() {
//             target.0 = cursor;
//             face(&mut transform, cursor);
//         }

//         // Abilities
//         let buttons = input.buttons();
//         let mut abilities_not_fired =
// abilities.iter().collect::<HashSet<_>>();         for ability in
// buttons.abilities_fired(abilities) {             let fired = ability.fire(
//                 &mut commands,
//                 &props,
//                 entity,
//                 &mut energy,
//                 &mut cooldowns,
//                 &transform,
//                 velocity,
//                 &target,
//                 ability_offset,
//                 foot_offset,
//                 &mut time_dilation,
//             );
//             if fired {
//                 abilities_not_fired.remove(&ability);
//             }
//         }

//         // Movement
//         desired_move.dir = input.movement().clamp_length_max(1.0);

//         // Menu
//         if buttons.contains(Action::Menu) {
//             pause_resume(&state, &mut next_state);
//         }
//     }
// }

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
