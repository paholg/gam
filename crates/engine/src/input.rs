use bevy_ecs::{
    entity::Entity,
    schedule::{NextState, State},
    system::{Commands, Query, Res, ResMut},
};
use bevy_rapier3d::prelude::{ExternalImpulse, Velocity};
use bevy_transform::components::Transform;
use bevy_utils::HashSet;

use crate::{
    ability::{properties::AbilityProps, Abilities},
    face,
    multiplayer::{Action, PlayerInputs},
    status_effect::{StatusEffect, StatusEffects},
    time::TickCounter,
    AppState, Cooldowns, Energy, MaxSpeed, Player, Target,
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
        &MaxSpeed,
        &mut ExternalImpulse,
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
        max_speed,
        mut impulse,
    ) in query.iter_mut()
    {
        let Some(input) = inputs.get(player) else {
            continue;
        };

        // Targeting
        if let Some(cursor) = input.cursor() {
            target.0 = cursor;
            face(&mut transform, cursor.extend(0.0));
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
        let dir = input.movement().clamp_length_max(1.0).extend(0.0);
        let mut max_impulse = max_speed.impulse;
        if status_effects
            .effects
            .contains(&StatusEffect::HyperSprinting)
        {
            max_impulse *= props.hyper_sprint.factor;
        }
        impulse.impulse += dir * max_impulse;

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
