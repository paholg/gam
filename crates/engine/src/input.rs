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
    movement::{DesiredMove, MaxSpeed},
    multiplayer::{Action, PlayerInputs},
    status_effect::{StatusEffect, StatusEffects},
    time::TickCounter,
    AppState, Cooldowns, Energy, Player, Target, To2d, To3d, UP,
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
    tick_counter: Res<TickCounter>,
    props: Res<AbilityProps>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut query: Query<(
        Entity,
        &mut Energy,
        &mut Cooldowns,
        &Velocity,
        &mut StatusEffects,
        &mut Transform,
        &Player,
        &mut Target,
        &Abilities,
        &mut MaxSpeed,
        &mut DesiredMove,
    )>,
) {
    for (
        entity,
        mut energy,
        mut cooldowns,
        velocity,
        mut status_effects,
        mut transform,
        player,
        mut target,
        abilities,
        mut max_speed,
        mut desired_move,
    ) in query.iter_mut()
    {
        let Some(input) = inputs.get(player) else {
            continue;
        };

        // Targeting
        if let Some(cursor) = input.cursor() {
            target.0 = cursor;
            if transform.translation.to_2d() != cursor {
                transform.look_at(cursor.to_3d(0.0), UP);
            }
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
                &mut status_effects,
                &target,
            );
            if fired {
                abilities_not_fired.remove(&ability);
            }
        }
        for ability in abilities_not_fired {
            ability.unfire(&mut commands, entity, &mut status_effects);
        }

        // Movement
        // TODO: Do this better
        *max_speed = MaxSpeed::default();
        if status_effects
            .effects
            .contains(&StatusEffect::HyperSprinting)
        {
            *max_speed *= props.hyper_sprint.factor;
        }
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
