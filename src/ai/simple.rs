use std::cmp::Ordering;

use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{Commands, Entity, Plugin, Quat, Query, Res, Transform, Vec3, With, Without},
};
use bevy_rapier3d::prelude::{ExternalImpulse, Velocity};
use rand::random;
use tracing::info;

use crate::{
    ability::{Ability, SHOT_SPEED},
    pointing_angle,
    time::TickCounter,
    Ai, Ally, Cooldowns, Enemy, FixedTimestepSystem, MaxSpeed,
};

pub struct SimpleAiPlugin;

impl Plugin for SimpleAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_engine_tick_system(update_enemy_orientation);
        app.add_engine_tick_system(update_ally_orientation);
        app.add_engine_tick_system(stupid_shoot_system);
        app.add_engine_tick_system(just_move_system);
    }
}

fn just_move_system(mut query: Query<(&mut ExternalImpulse, &Transform, &MaxSpeed), With<Ai>>) {
    for (mut impulse, transform, max_speed) in query.iter_mut() {
        let target_dir = transform.rotation * Vec3::Y;
        let randx = random::<f32>() - 0.5;
        let randy = random::<f32>() - 0.5;
        let new_target = target_dir + Vec3::new(randx, randy, 0.0);

        impulse.impulse = new_target.normalize() * max_speed.impulse;
    }
}

fn point_to_closest<T: ReadOnlyWorldQuery, U: ReadOnlyWorldQuery>(
    mut query: Query<&mut Transform, T>,
    targets: Query<(&Transform, &Velocity), U>,
) {
    for mut transform in query.iter_mut() {
        let closest_target = targets
            .iter()
            .map(|(t, v)| (t, v, transform.translation.distance(t.translation)))
            .min_by(|(_, _, d1), (_, _, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        if let Some((trans, vel, dist)) = closest_target {
            let dt = dist / SHOT_SPEED;
            let lead_translation = trans.translation + vel.linvel * dt;
            info!(%trans.translation, %lead_translation);
            let angle = pointing_angle(transform.translation, lead_translation);
            if !angle.is_nan() {
                transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
            }
        }
    }
}

fn update_enemy_orientation(
    ally_query: Query<(&Transform, &Velocity), (With<Ally>, Without<Enemy>)>,
    enemy_query: Query<&mut Transform, (With<Enemy>, With<Ai>, Without<Ally>)>,
) {
    point_to_closest(enemy_query, ally_query);
}

fn update_ally_orientation(
    enemy_query: Query<(&Transform, &Velocity), (With<Enemy>, Without<Ally>)>,
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
