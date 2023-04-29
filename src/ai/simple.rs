use std::cmp::Ordering;

use bevy::{
    ecs::query::ReadOnlyWorldQuery,
    prelude::{
        Commands, Component, Entity, Plugin, Quat, Query, Res, Transform, Vec2, Vec3, With, Without,
    },
};
use bevy_rapier3d::prelude::{ExternalImpulse, Velocity};
use rand::Rng;

use crate::{
    ability::{Ability, SHOT_SPEED},
    pointing_angle,
    status_effect::StatusEffects,
    system::point_in_plane,
    time::TickCounter,
    Ai, Ally, Cooldowns, Enemy, Energy, FixedTimestepSystem, MaxSpeed,
};

pub struct SimpleAiPlugin;

#[derive(Component)]
pub enum Attitude {
    Chase,
    RunAway,
    PickPoint(Vec3),
}

impl Attitude {
    pub fn rand() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => Self::Chase,
            1 => Self::RunAway,
            _ => Self::PickPoint(point_in_plane()),
        }
    }
}

impl Plugin for SimpleAiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_engine_tick_system(update_enemy_orientation);
        app.add_engine_tick_system(update_ally_orientation);
        app.add_engine_tick_system(stupid_shoot_system);
        app.add_engine_tick_system(just_move_system);
    }
}

fn just_move_system(
    mut query: Query<(&mut ExternalImpulse, &Transform, &MaxSpeed, &mut Attitude), With<Ai>>,
) {
    for (mut impulse, transform, max_speed, mut attitude) in query.iter_mut() {
        let target_vec = match *attitude {
            Attitude::Chase => transform.rotation * Vec3::Y,
            Attitude::RunAway => -(transform.rotation * Vec3::Y),
            Attitude::PickPoint(ref mut target) => {
                while transform.translation.distance_squared(*target) < 1.0 {
                    *target = point_in_plane();
                }
                *target - transform.translation
            }
        };
        impulse.impulse = target_vec.normalize() * max_speed.impulse;
    }
}

fn point_to_closest<T: ReadOnlyWorldQuery, U: ReadOnlyWorldQuery>(
    mut query: Query<(&mut Transform, &Velocity), T>,
    targets: Query<(&Transform, &Velocity), U>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        let closest_target = targets
            .iter()
            .map(|(t, v)| (t, v, transform.translation.distance(t.translation)))
            .min_by(|(_, _, d1), (_, _, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        if let Some((trans, vel, dist)) = closest_target {
            let dt = dist / SHOT_SPEED;
            let lead = (vel.linvel - velocity.linvel) * dt * 0.5; // Just partially lead for now
            let lead_translation = trans.translation + lead;
            let angle = pointing_angle(transform.translation, lead_translation);
            if !angle.is_nan() {
                transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);
            }
        }
    }
}

fn update_enemy_orientation(
    ally_query: Query<(&Transform, &Velocity), (With<Ally>, Without<Enemy>)>,
    enemy_query: Query<(&mut Transform, &Velocity), (With<Enemy>, With<Ai>, Without<Ally>)>,
) {
    point_to_closest(enemy_query, ally_query);
}

fn update_ally_orientation(
    enemy_query: Query<(&Transform, &Velocity), (With<Enemy>, Without<Ally>)>,
    ally_query: Query<(&mut Transform, &Velocity), (With<Ally>, With<Ai>, Without<Enemy>)>,
) {
    point_to_closest(ally_query, enemy_query);
}

fn stupid_shoot_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,

    mut q_ai: Query<
        (
            Entity,
            &mut Cooldowns,
            &mut Energy,
            &Velocity,
            &Transform,
            &mut StatusEffects,
        ),
        With<Ai>,
    >,
) {
    for (entity, mut cooldowns, mut energy, velocity, transform, mut status_effects) in
        q_ai.iter_mut()
    {
        Ability::Shoot.fire(
            false,
            &mut commands,
            &tick_counter,
            entity,
            &mut energy,
            &mut cooldowns,
            transform,
            velocity,
            &mut status_effects,
            Vec2::ZERO,
        );
    }
}
