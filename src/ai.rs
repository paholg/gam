use bevy::prelude::{
    Assets, Commands, Component, Entity, Mesh, Plugin, Quat, Query, ResMut, StandardMaterial,
    Transform, Vec3, With, Without,
};
use bevy_rapier3d::prelude::Velocity;
use big_brain::{
    prelude::ActionState, scorers::Score, thinker::Actor, BigBrainPlugin, BigBrainStage,
};

use crate::{ability::Ability, pointing_angle, Enemy, MaxSpeed, Player, PlayerCooldowns};

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update_enemy_orientation)
            .add_plugin(BigBrainPlugin)
            .add_system_to_stage(BigBrainStage::Actions, shot_action_system)
            .add_system_to_stage(BigBrainStage::Scorers, shot_scorer_system);
    }
}

fn update_enemy_orientation(
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut Transform, &mut Velocity), (With<Enemy>, Without<Player>)>,
) {
    if let Some(player_transform) = player_query.iter().next() {
        enemy_query.for_each_mut(|(mut transform, _velocity)| {
            let angle = pointing_angle(transform.translation, player_transform.translation);
            if !angle.is_nan() {
                transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
            }
        });
    }
}

#[derive(Debug, Clone, Component)]
pub struct ShotScorer;

fn shot_scorer_system(mut query: Query<(&Actor, &mut Score), With<ShotScorer>>) {
    for (Actor(_actor), mut score) in query.iter_mut() {
        score.set(1.0);
    }
}

#[derive(Debug, Clone, Component)]
pub struct ShotAction;

fn shot_action_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut query: Query<(&Actor, &mut ActionState), With<ShotAction>>,
    mut q_enemy: Query<
        (
            Entity,
            &mut PlayerCooldowns,
            &Velocity,
            &mut MaxSpeed,
            &Transform,
        ),
        With<Enemy>,
    >,
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
                        &velocity,
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
