use std::time::Duration;

use bevy::prelude::{
    shape::Icosphere, Assets, Color, Commands, Component, ComputedVisibility, Entity,
    GlobalTransform, Mesh, Query, Res, ResMut, StandardMaterial, Transform, Vec3, Visibility, With,
    Without,
};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RapierContext, RigidBody, Sensor, Velocity};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    ai::qlearning::{AllyAi, EnemyAi},
    time::{Tick, TickCounter},
    Ally, Cooldowns, Enemy, Health, MaxSpeed, Object, PLAYER_R,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Shoot,
}

impl Ability {
    #[allow(clippy::too_many_arguments)]
    pub fn fire(
        &self,
        commands: &mut Commands,
        #[cfg(feature = "graphics")] meshes: &mut ResMut<Assets<Mesh>>,
        #[cfg(feature = "graphics")] materials: &mut ResMut<Assets<StandardMaterial>>,
        entity: Entity,
        cooldowns: &mut Cooldowns,
        max_speed: &mut MaxSpeed,
        transform: &Transform,
        velocity: &Velocity,
    ) -> bool {
        match self {
            Ability::None => true,
            Ability::HyperSprint => hyper_sprint(commands, entity, cooldowns, max_speed),
            Ability::Shoot => shoot(
                commands,
                cooldowns,
                #[cfg(feature = "graphics")]
                meshes,
                #[cfg(feature = "graphics")]
                materials,
                transform,
                velocity,
            ),
        }
    }
}

#[derive(Component)]
pub struct HyperSprinting {
    duration: Tick,
}

const HYPER_SPRINT_FACTOR: f32 = 5.0;
pub const HYPER_SPRINT_COOLDOWN: Tick = Tick::new(Duration::new(5, 0));
const HYPER_SPRINT_DURATION: Tick = Tick::new(Duration::from_secs_f32(0.15));

fn hyper_sprint(
    commands: &mut Commands,
    entity: Entity,
    cooldowns: &mut Cooldowns,
    max_speed: &mut MaxSpeed,
) -> bool {
    if cooldowns.hyper_sprint.is_zero() {
        cooldowns.hyper_sprint = HYPER_SPRINT_COOLDOWN;
        max_speed.0 *= HYPER_SPRINT_FACTOR;
        commands.entity(entity).insert(HyperSprinting {
            duration: HYPER_SPRINT_DURATION,
        });
        true
    } else {
        false
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HyperSprinting, &mut MaxSpeed)>,
) {
    for (entity, mut hyper_sprinting, mut max_speed) in query.iter_mut() {
        if hyper_sprinting.duration.tick().is_zero() {
            max_speed.0 /= HYPER_SPRINT_FACTOR;
            commands.entity(entity).remove::<HyperSprinting>();
        }
    }
}

pub const SHOOT_COOLDOWN: Tick = Tick::new(Duration::from_millis(100));
const SHOT_DURATION: Tick = Tick::new(Duration::from_secs(2));
const SHOT_SPEED: f32 = 50.0;
const SHOT_R: f32 = 0.1;
const SHOT_DAMAGE: f32 = 21.0;

#[derive(Component)]
pub struct Shot {
    duration: Tick,
}

fn shoot(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    #[cfg(feature = "graphics")] meshes: &mut ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] materials: &mut ResMut<Assets<StandardMaterial>>,
    transform: &Transform,
    velocity: &Velocity,
) -> bool {
    if cooldowns.shoot.is_zero() {
        cooldowns.shoot = SHOOT_COOLDOWN;

        let dir = transform.rotation * Vec3::Y;
        let position = transform.translation + dir * (PLAYER_R + SHOT_R);
        let vel = velocity.linvel + dir.truncate() * SHOT_SPEED;
        commands.spawn((
            Object {
                #[cfg(feature = "graphics")]
                material: materials.add(Color::BLUE.into()),
                #[cfg(feature = "graphics")]
                mesh: meshes.add(Mesh::from(Icosphere {
                    radius: SHOT_R,
                    subdivisions: 5,
                })),
                transform: Transform::from_translation(position),
                global_transform: GlobalTransform::default(),
                #[cfg(feature = "graphics")]
                visibility: Visibility::VISIBLE,
                #[cfg(feature = "graphics")]
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(SHOT_R),
                body: RigidBody::Dynamic,
                velocity: Velocity {
                    linvel: vel,
                    angvel: 0.0,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
            Sensor,
            Shot {
                duration: SHOT_DURATION,
            },
        ));
        true
    } else {
        false
    }
}

pub fn shot_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Shot)>,
    tick_counter: Res<TickCounter>,
) {
    if tick_counter.diagnostic_iter() {
        let num_shots = query.iter().count();
        info!(%num_shots, "Shots fired!");
    }
    for (entity, mut shot) in query.iter_mut() {
        if shot.duration.tick().is_zero() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn shot_hit_system_ally(
    // TODO: This is a temporary hack for Ai tracking of damage done.
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<Entity, With<Shot>>,
    mut target_query: Query<(Entity, &mut Health), With<Ally>>,
    mut ally_ai: ResMut<AllyAi>,
    mut enemy_ai: ResMut<EnemyAi>,
) {
    for shot in shot_query.iter() {
        for (target, mut health) in target_query.iter_mut() {
            if rapier_context.intersection_pair(shot, target) == Some(true) {
                commands.entity(shot).despawn();
                health.cur -= SHOT_DAMAGE;
                ally_ai.take_damage(SHOT_DAMAGE);
                enemy_ai.do_damage(SHOT_DAMAGE);
            }
        }
    }
}

pub fn shot_hit_system_enemy(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<Entity, With<Shot>>,
    mut target_query: Query<(Entity, &mut Health), With<Enemy>>,
    mut ally_ai: ResMut<AllyAi>,
    mut enemy_ai: ResMut<EnemyAi>,
) {
    for shot in shot_query.iter() {
        for (target, mut health) in target_query.iter_mut() {
            if rapier_context.intersection_pair(shot, target) == Some(true) {
                commands.entity(shot).despawn();
                health.cur -= SHOT_DAMAGE;
                ally_ai.do_damage(SHOT_DAMAGE);
                enemy_ai.take_damage(SHOT_DAMAGE);
            }
        }
    }
}

pub fn shot_miss_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<Entity, With<Shot>>,
    target_query: Query<Entity, Without<Health>>,
) {
    for shot in shot_query.iter() {
        for (other_entity, _, intersecting) in rapier_context.intersections_with(shot) {
            if intersecting && target_query.get(other_entity).is_ok() {
                commands.entity(shot).despawn();
            }
        }
    }
}
