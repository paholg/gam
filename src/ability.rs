use std::time::Duration;

use bevy::prelude::{
    shape::Icosphere, Assets, Color, Commands, Component, ComputedVisibility, Entity,
    GlobalTransform, Mesh, Quat, Query, Res, ResMut, StandardMaterial, Transform, Vec3, Visibility,
    With, Without,
};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RapierContext, RigidBody, Sensor, Velocity};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tracing::{info, warn};

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

pub fn shot_despawn_system(mut commands: Commands, mut query: Query<(Entity, &mut Shot)>) {
    for (entity, mut shot) in query.iter_mut() {
        if shot.duration.tick().is_zero() {
            commands.entity(entity).despawn();
        }
    }
}

// Note: This iterates through all intersection_pairs. We should use one system
// for all such intersections to avoid duplicate work.
pub fn shot_hit_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<Entity, With<Shot>>,
    mut ally_query: Query<&mut Health, (With<Ally>, Without<Enemy>)>,
    mut enemy_query: Query<&mut Health, (With<Enemy>, Without<Ally>)>,
    miss_query: Query<Entity, Without<Health>>,
    // TODO: This is a temporary hack for Ai tracking of damage done.
    mut ally_ai: ResMut<AllyAi>,
    mut enemy_ai: ResMut<EnemyAi>,
) {
    let mut shots_to_despawn: SmallVec<[Entity; 10]> = smallvec::SmallVec::new();
    for (entity1, entity2, intersecting) in rapier_context.intersection_pairs() {
        if intersecting {
            let (shot_entity, target_entity) = if shot_query.get(entity1).is_ok() {
                (entity1, entity2)
            } else if shot_query.get(entity2).is_ok() {
                (entity2, entity1)
            } else {
                continue;
            };

            if let Ok(mut health) = ally_query.get_mut(target_entity) {
                health.cur -= SHOT_DAMAGE;
                ally_ai.take_damage(SHOT_DAMAGE);
                enemy_ai.do_damage(SHOT_DAMAGE);
                shots_to_despawn.push(shot_entity);
            }
            if let Ok(mut health) = enemy_query.get_mut(target_entity) {
                health.cur -= SHOT_DAMAGE;
                ally_ai.do_damage(SHOT_DAMAGE);
                enemy_ai.take_damage(SHOT_DAMAGE);
                shots_to_despawn.push(shot_entity);
            }
            if miss_query.get(target_entity).is_ok() {
                shots_to_despawn.push(shot_entity);
            }
        }
    }
    shots_to_despawn.sort();
    shots_to_despawn.dedup();

    for shot in shots_to_despawn.into_iter() {
        commands.entity(shot).despawn();
    }
}
