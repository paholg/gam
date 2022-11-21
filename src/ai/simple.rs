use std::cmp::Ordering;

use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{
        Added, Assets, Commands, Component, Entity, Mesh, Plugin, Quat, Query, ResMut,
        StandardMaterial, Transform, Vec3, With, Without,
    },
};
use bevy_rapier2d::prelude::Velocity;
use big_brain::{
    prelude::{ActionState, FirstToScore},
    scorers::Score,
    thinker::{Actor, Thinker},
    BigBrainPlugin, BigBrainStage,
};

use crate::{
    ability::Ability, pointing_angle, Ai, Ally, Cooldowns, Enemy, FixedTimestepSystem, MaxSpeed,
};

pub struct SimpleAiPlugin;

impl Plugin for SimpleAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_engine_tick_system(add_ai_system)
            .add_engine_tick_system(update_enemy_orientation)
            .add_engine_tick_system(update_ally_orientation)
            // TODO: These should tick with the engine.
            .add_plugin(BigBrainPlugin)
            .add_system_to_stage(BigBrainStage::Actions, shot_action_system)
            .add_system_to_stage(BigBrainStage::Scorers, shot_scorer_system);
    }
}

fn point_to_closest<T: ReadOnlyWorldQuery, U: ReadOnlyWorldQuery>(
    mut query: Query<&mut Transform, T>,
    targets: Query<&Transform, U>,
) {
    query.for_each_mut(|mut transform| {
        let closest_target = targets
            .iter()
            .map(|target_transform| {
                (
                    target_transform,
                    transform.translation.distance(target_transform.translation),
                )
            })
            .min_by(|(_, d1), (_, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal))
            .map(|(transform, _)| transform.translation);
        if let Some(closest_target) = closest_target {
            let angle = pointing_angle(transform.translation, closest_target);
            if !angle.is_nan() {
                transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
            }
        }
    })
}

fn update_enemy_orientation(
    ally_query: Query<&Transform, (With<Ally>, Without<Enemy>)>,
    enemy_query: Query<&mut Transform, (With<Enemy>, With<Ai>, Without<Ally>)>,
) {
    point_to_closest(enemy_query, ally_query);
}

fn update_ally_orientation(
    enemy_query: Query<&Transform, (With<Enemy>, Without<Ally>)>,
    ally_query: Query<&mut Transform, (With<Ally>, With<Ai>, Without<Enemy>)>,
) {
    point_to_closest(ally_query, enemy_query);
}

#[derive(Debug, Clone, Component)]
pub struct ShotScorer;

fn shot_scorer_system(mut query: Query<(&Actor, &mut Score), With<ShotScorer>>) {
    for (Actor(_actor), mut score) in query.iter_mut() {
        score.set(0.8);
    }
}

#[derive(Debug, Clone, Component)]
pub struct ShotAction;

fn shot_action_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut query: Query<(&Actor, &mut ActionState), With<ShotAction>>,
    mut q_enemy: Query<(Entity, &mut Cooldowns, &Velocity, &mut MaxSpeed, &Transform), With<Ai>>,
) {
    // for (Actor(actor), mut state, entity, mut cooldowns, velocity, mut max_speed, transform) in
    for (Actor(actor), mut state) in query.iter_mut() {
        if let Ok((entity, mut cooldowns, velocity, mut max_speed, transform)) =
            q_enemy.get_mut(*actor)
        {
            match *state {
                ActionState::Requested => {
                    if Ability::Shoot.fire(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        entity,
                        &mut cooldowns,
                        &mut max_speed,
                        transform,
                        velocity,
                    ) {
                        *state = ActionState::Success;
                    } else {
                        *state = ActionState::Failure;
                    }
                }
                ActionState::Cancelled => *state = ActionState::Failure,
                _ => {}
            }
        }
    }
}

fn add_ai_system(mut commands: Commands, ai_query: Query<Entity, Added<Ai>>) {
    for entity in ai_query.iter() {
        commands.entity(entity).insert((Thinker::build()
            .picker(FirstToScore { threshold: 0.8 })
            .when(ShotScorer, ShotAction),));
    }
}
