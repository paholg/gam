use std::cmp::Ordering;

use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{Commands, Entity, Plugin, Quat, Query, Res, Transform, Vec3, With, Without},
};
use bevy_rapier3d::prelude::Velocity;

use crate::{
    ability::Ability, pointing_angle, time::TickCounter, Ai, Ally, Cooldowns, Enemy,
    FixedTimestepSystem, MaxSpeed,
};

pub struct SimpleAiPlugin;

impl Plugin for SimpleAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_engine_tick_system(update_enemy_orientation);
        app.add_engine_tick_system(update_ally_orientation);
        app.add_engine_tick_system(stupid_shoot_system);
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

fn stupid_shoot_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,

    mut q_ai: Query<(Entity, &mut Cooldowns, &Velocity, &mut MaxSpeed, &Transform), With<Ai>>,
) {
    for (entity, mut cooldowns, velocity, mut max_speed, transform) in q_ai.iter_mut() {
        Ability::Shoot.fire(
            &mut commands,
            &tick_counter,
            entity,
            &mut cooldowns,
            &mut max_speed,
            transform,
            velocity,
        );
    }
}
