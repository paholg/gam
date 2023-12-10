use bevy_ecs::{
    entity::Entity,
    schedule::{NextState, State},
    system::{Commands, Query, Res, ResMut},
};
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;
use bevy_utils::HashSet;

use crate::{
    ability::{properties::AbilityProps, Abilities},
    face,
    movement::DesiredMove,
    multiplayer::{Action, PlayerInputs},
    status_effect::TimeDilation,
    time::FrameCounter,
    AbilityOffset, AppState, Cooldowns, Energy, Player, Target,
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

        if buttons.contains(Action::Menu) {
            pause_resume(&state, &mut next_state);
        }
    }
}

pub fn apply_inputs(
    inputs: Res<PlayerInputs>,
    mut commands: Commands,
    tick_counter: Res<FrameCounter>,
    props: Res<AbilityProps>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<(
        Entity,
        &mut Energy,
        &mut Cooldowns,
        &Velocity,
        &mut Transform,
        &Player,
        &mut Target,
        &Abilities,
        &mut DesiredMove,
        &AbilityOffset,
        &mut TimeDilation,
    )>,
) {
    for (
        entity,
        mut energy,
        mut cooldowns,
        velocity,
        mut transform,
        player,
        mut target,
        abilities,
        mut desired_move,
        ability_offset,
        mut time_dilation,
    ) in query.iter_mut()
    {
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
        let mut abilities_not_fired = abilities.iter().collect::<HashSet<_>>();
        for ability in buttons.abilities_fired(abilities) {
            let fired = ability.fire(
                &mut commands,
                &tick_counter,
                &props,
                entity,
                &mut energy,
                &mut cooldowns,
                &transform,
                velocity,
                &target,
                ability_offset,
                &mut time_dilation,
            );
            if fired {
                abilities_not_fired.remove(&ability);
            }
        }

        // Movement
        desired_move.dir = input.movement().clamp_length_max(1.0);

        // Menu
        if buttons.contains(Action::Menu) {
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
