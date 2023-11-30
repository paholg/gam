use std::cmp::Ordering;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{ReadOnlyWorldQuery, With, Without},
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Commands, Query, Res},
};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;
use rand::Rng;

use crate::{
    ability::{properties::AbilityProps, Ability},
    face,
    level::LevelProps,
    movement::DesiredMove,
    status_effect::StatusEffects,
    time::TickCounter,
    AbilityOffset, Ai, Ally, Cooldowns, Enemy, Energy, Target, To2d, FORWARD,
};

#[derive(Component)]
pub enum Attitude {
    Chase,
    RunAway,
    PickPoint(Vec3),
}

impl Attitude {
    pub fn rand(level: &LevelProps) -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => Self::Chase,
            1 => Self::RunAway,
            _ => Self::PickPoint(level.point_in_plane()),
        }
    }
}

pub fn system_set() -> SystemConfigs {
    (
        update_enemy_orientation,
        update_ally_orientation,
        stupid_gun_system,
        just_move_system,
    )
        .chain()
}

fn just_move_system(
    mut query: Query<(&mut DesiredMove, &Transform, &mut Attitude), With<Ai>>,
    level: Res<LevelProps>,
) {
    for (mut desired_move, transform, mut attitude) in query.iter_mut() {
        let dir = match *attitude {
            Attitude::Chase => transform.rotation * FORWARD,
            Attitude::RunAway => -(transform.rotation * FORWARD),
            Attitude::PickPoint(ref mut target) => {
                while transform.translation.distance_squared(*target) < 1.0 {
                    *target = level.point_in_plane();
                }
                *target - transform.translation
            }
        }
        .to_2d();

        desired_move.dir = dir;
    }
}

fn point_to_closest<T: ReadOnlyWorldQuery, U: ReadOnlyWorldQuery>(
    mut query: Query<(&mut Transform, &Velocity), T>,
    targets: Query<(&Transform, &Velocity), U>,
    shot_speed: f32,
) {
    for (mut transform, velocity) in query.iter_mut() {
        let closest_target = targets
            .iter()
            .map(|(t, v)| (t, v, transform.translation.distance(t.translation)))
            .min_by(|(_, _, d1), (_, _, d2)| d1.partial_cmp(d2).unwrap_or(Ordering::Equal));
        if let Some((trans, vel, dist)) = closest_target {
            let dt = dist / shot_speed;
            let lead = (vel.linvel - velocity.linvel) * dt * 0.5; // Just partially lead for now
            let lead_translation = trans.translation + lead;

            face(&mut transform, lead_translation.to_2d());
        }
    }
}

fn update_enemy_orientation(
    ally_query: Query<(&Transform, &Velocity), (With<Ally>, Without<Enemy>)>,
    enemy_query: Query<(&mut Transform, &Velocity), (With<Enemy>, With<Ai>, Without<Ally>)>,
    props: Res<AbilityProps>,
) {
    let shot_speed = props.gun.speed;
    point_to_closest(enemy_query, ally_query, shot_speed);
}

fn update_ally_orientation(
    enemy_query: Query<(&Transform, &Velocity), (With<Enemy>, Without<Ally>)>,
    ally_query: Query<(&mut Transform, &Velocity), (With<Ally>, With<Ai>, Without<Enemy>)>,
    props: Res<AbilityProps>,
) {
    let shot_speed = props.gun.speed;
    point_to_closest(ally_query, enemy_query, shot_speed);
}

fn stupid_gun_system(
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
            &AbilityOffset,
        ),
        With<Ai>,
    >,
    props: Res<AbilityProps>,
) {
    for (
        entity,
        mut cooldowns,
        mut energy,
        velocity,
        transform,
        mut status_effects,
        ability_offset,
    ) in q_ai.iter_mut()
    {
        Ability::Gun.fire(
            &mut commands,
            &tick_counter,
            &props,
            entity,
            &mut energy,
            &mut cooldowns,
            transform,
            velocity,
            &mut status_effects,
            &Target::default(),
            ability_offset,
        );
    }
}
