use bevy_app::Plugin;
use bevy_ecs::{
    entity::Entity,
    schedule::{NextState, State},
    system::{Commands, Query, Res, ResMut},
};
use bevy_math::{Quat, Vec3};
use bevy_rapier3d::prelude::{ExternalImpulse, RapierConfiguration, Velocity};
use bevy_transform::components::Transform;

use crate::{
    ability::{properties::AbilityProps, Abilities},
    multiplayer::{Action, PlayerInputs},
    pointing_angle,
    status_effect::{StatusEffect, StatusEffects},
    time::TickCounter,
    AppState, Cooldowns, Energy, EngineTickSystem, MaxSpeed, Player, Target,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_engine_tick_systems(apply_inputs);
    }
}

fn apply_inputs(
    inputs: Res<PlayerInputs>,
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    props: Res<AbilityProps>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut physics_config: ResMut<RapierConfiguration>,
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
            return;
        };

        // Targeting
        let cursor = input.cursor();
        target.0 = cursor;
        let angle = pointing_angle(transform.translation, cursor.extend(0.0));
        transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

        // Movement
        let dir = input.movement().clamp_length_max(1.0).extend(0.0);
        let mut max_impulse = max_speed.impulse;
        if status_effects
            .effects
            .contains(&StatusEffect::HyperSprinting)
        {
            max_impulse *= props.hyper_sprint.factor;
        }
        impulse.impulse = dir * max_impulse;

        // Abilities
        let buttons = input.buttons();

        for ability in buttons.abilities_fired(abilities) {
            ability.fire(
                true, // FIXME
                &mut commands,
                &tick_counter,
                &props,
                entity,
                &mut energy,
                &mut cooldowns,
                &*transform,
                &velocity,
                &mut status_effects,
                &target,
            );
        }

        // Menu
        if buttons.contains(Action::Menu) {
            match state.get() {
                AppState::Loading => (),
                AppState::Running => {
                    physics_config.physics_pipeline_active = false;
                    next_state.set(AppState::Paused);
                }
                AppState::Paused => {
                    physics_config.physics_pipeline_active = true;
                    next_state.set(AppState::Running);
                }
            }
        }
    }
}
