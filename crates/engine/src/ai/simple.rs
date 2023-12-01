use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Commands, Query, Res},
};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{RapierContext, Velocity};
use bevy_transform::components::Transform;
use rand::Rng;

use crate::{
    ability::{properties::AbilityProps, Ability},
    level::LevelProps,
    movement::DesiredMove,
    status_effect::StatusEffects,
    time::TickCounter,
    AbilityOffset, Ai, Cooldowns, Energy, Target, To2d, FORWARD,
};

use super::{update_ally_orientation, update_enemy_orientation};

#[derive(Component)]
pub struct SimpleAi;

#[derive(Component)]
pub enum Attitude {
    Chase,
    RunAway,
    PickPoint(Vec3),
}

impl Attitude {
    pub fn rand(level: &LevelProps, rapier_context: &RapierContext) -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => Self::Chase,
            1 => Self::RunAway,
            _ => Self::PickPoint(level.point_in_plane(rapier_context)),
        }
    }
}

pub fn system_set() -> SystemConfigs {
    (
        update_enemy_orientation::<SimpleAi>,
        update_ally_orientation::<SimpleAi>,
        stupid_gun_system,
        just_move_system,
    )
        .chain()
}

fn just_move_system(
    mut query: Query<(&mut DesiredMove, &Transform, &mut Attitude), With<SimpleAi>>,
    level: Res<LevelProps>,
    rapier_context: Res<RapierContext>,
) {
    for (mut desired_move, transform, mut attitude) in query.iter_mut() {
        let dir = match *attitude {
            Attitude::Chase => transform.rotation * FORWARD,
            Attitude::RunAway => -(transform.rotation * FORWARD),
            Attitude::PickPoint(ref mut target) => {
                while transform.translation.distance_squared(*target) < 1.0 {
                    *target = level.point_in_plane(&rapier_context);
                }
                *target - transform.translation
            }
        }
        .to_2d()
        .normalize_or_zero();

        desired_move.dir = dir;
    }
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
