use std::{f32::consts::PI, time::Duration};

use bevy::prelude::{
    Added, Commands, Component, Entity, EventReader, EventWriter, GlobalTransform, Quat, Query,
    Res, Transform, Vec3, Without,
};

use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, ColliderMassProperties, CollisionEvent, LockedAxes,
    ReadMassProperties, RigidBody, Sensor, Velocity,
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use tracing::info;

use crate::{
    time::{Tick, TickCounter},
    Cooldowns, Health, MaxSpeed, Object, PLAYER_R,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Shoot,
    Shotgun,
}

impl Ability {
    #[allow(clippy::too_many_arguments)]
    pub fn fire(
        &self,
        commands: &mut Commands,
        tick_counter: &TickCounter,
        entity: Entity,
        cooldowns: &mut Cooldowns,
        max_speed: &mut MaxSpeed,
        transform: &Transform,
        velocity: &Velocity,
    ) -> bool {
        match self {
            Ability::None => true,
            Ability::HyperSprint => {
                hyper_sprint(commands, tick_counter, entity, cooldowns, max_speed)
            }
            Ability::Shoot => shoot(
                commands,
                cooldowns,
                tick_counter,
                transform,
                velocity,
                entity,
            ),
            Ability::Shotgun => shotgun(
                commands,
                cooldowns,
                tick_counter,
                transform,
                velocity,
                entity,
            ),
        }
    }
}

#[derive(Component)]
pub struct HyperSprinting {
    duration: Tick,
}

const HYPER_SPRINT_FACTOR: f32 = 7.0;
pub const HYPER_SPRINT_COOLDOWN: Tick = Tick::new(Duration::new(3, 0));
const HYPER_SPRINT_DURATION: Tick = Tick::new(Duration::from_secs_f32(0.15));

fn hyper_sprint(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    entity: Entity,
    cooldowns: &mut Cooldowns,
    max_speed: &mut MaxSpeed,
) -> bool {
    if cooldowns.hyper_sprint.before_now(tick_counter) {
        cooldowns.hyper_sprint = tick_counter.at(HYPER_SPRINT_COOLDOWN);
        max_speed.max_speed *= HYPER_SPRINT_FACTOR;
        max_speed.impulse *= HYPER_SPRINT_FACTOR;
        commands.entity(entity).insert(HyperSprinting {
            duration: tick_counter.at(HYPER_SPRINT_DURATION),
        });
        true
    } else {
        false
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<(Entity, &HyperSprinting, &mut MaxSpeed, &Transform)>,
) {
    for (entity, hyper_sprinting, mut max_speed, _sprinter_transform) in query.iter_mut() {
        if hyper_sprinting.duration.before_now(&tick_counter) {
            max_speed.max_speed /= HYPER_SPRINT_FACTOR;
            max_speed.impulse /= HYPER_SPRINT_FACTOR;
            commands.entity(entity).remove::<HyperSprinting>();
        }
    }
}

pub const SHOOT_COOLDOWN: Tick = Tick::new(Duration::from_millis(250));
const SHOT_DURATION: Tick = Tick::new(Duration::from_secs(10));
pub const SHOT_SPEED: f32 = 30.0;
pub const SHOT_R: f32 = 0.15;
pub const ABILITY_Z: f32 = 0.0;
const SHOT_DAMAGE: f32 = 1.0;

#[derive(Component)]
pub struct Shot {
    shooter: Entity,
    duration: Tick,
    damage: f32,
}

fn shoot(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    tick_counter: &TickCounter,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) -> bool {
    if cooldowns.shoot.before_now(tick_counter) {
        cooldowns.shoot = tick_counter.at(SHOOT_COOLDOWN);

        let dir = transform.rotation * Vec3::Y;
        let position =
            transform.translation + dir * (PLAYER_R + SHOT_R + 0.01) + ABILITY_Z * Vec3::Z;
        let vel = dir * SHOT_SPEED + velocity.linvel;
        commands.spawn((
            Object {
                transform: Transform::from_translation(position)
                    .with_scale(Vec3::new(SHOT_R, SHOT_R, SHOT_R)),
                global_transform: GlobalTransform::default(),
                collider: Collider::ball(SHOT_R),
                mass_props: ColliderMassProperties::Density(10000.0),
                body: RigidBody::Dynamic,
                velocity: Velocity {
                    linvel: vel,
                    angvel: Vec3::ZERO,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                mass: ReadMassProperties::default(),
            },
            Sensor,
            Shot {
                duration: tick_counter.at(SHOT_DURATION),
                shooter,
                damage: SHOT_DAMAGE,
            },
            Ccd::enabled(),
            ActiveEvents::COLLISION_EVENTS,
        ));
        true
    } else {
        false
    }
}

pub const SHOTGUN_COOLDOWN: Tick = Tick::new(Duration::from_millis(750));
const SHOTGUN_DURATION: Tick = Tick::new(Duration::from_secs(10));
pub const SHOTGUN_SPEED: f32 = 30.0;
pub const SHOTGUN_R: f32 = 0.15;
const SHOTGUN_DAMAGE: f32 = 1.0;
const N_PELLETS: usize = 8;
const SPREAD: f32 = PI * 0.125; // Spread angle in radians

fn shotgun(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    tick_counter: &TickCounter,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) -> bool {
    if cooldowns.shotgun.before_now(tick_counter) {
        cooldowns.shotgun = tick_counter.at(SHOTGUN_COOLDOWN);

        for i in 0..N_PELLETS {
            let idx = i as f32;
            let n_pellets = N_PELLETS as f32;
            let relative_angle = (n_pellets * 0.5 - idx) / n_pellets * SPREAD;
            let relative_angle = Quat::from_rotation_z(relative_angle);
            let dir = (transform.rotation * relative_angle) * Vec3::Y;
            let position =
                transform.translation + dir * (PLAYER_R + SHOTGUN_R + 0.01) + ABILITY_Z * Vec3::Z;
            let vel = dir * SHOTGUN_SPEED + velocity.linvel;
            commands.spawn((
                Object {
                    transform: Transform::from_translation(position)
                        .with_scale(Vec3::new(SHOTGUN_R, SHOTGUN_R, SHOTGUN_R)),
                    global_transform: GlobalTransform::default(),
                    collider: Collider::ball(SHOTGUN_R),
                    mass_props: ColliderMassProperties::Density(10000.0),
                    body: RigidBody::Dynamic,
                    velocity: Velocity {
                        linvel: vel,
                        angvel: Vec3::ZERO,
                    },
                    locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                    mass: ReadMassProperties::default(),
                },
                Sensor,
                Shot {
                    duration: tick_counter.at(SHOTGUN_DURATION),
                    shooter,
                    damage: SHOTGUN_DAMAGE,
                },
                Ccd::enabled(),
                ActiveEvents::COLLISION_EVENTS,
            ));
        }
        true
    } else {
        false
    }
}

pub fn shot_kickback_system(
    shot_query: Query<(&Velocity, &ReadMassProperties, &Shot), Added<Shot>>,
    mut momentum_query: Query<(&mut Velocity, &ReadMassProperties), Without<Shot>>,
) {
    for (v, m, shot) in shot_query.iter() {
        let Ok((mut shooter_v, shooter_m)) = momentum_query.get_mut(shot.shooter) else { continue ; };
        shooter_v.linvel -= v.linvel * m.0.mass / shooter_m.0.mass;
    }
}

pub fn shot_despawn_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<(Entity, &mut Shot)>,
) {
    for (entity, shot) in query.iter_mut() {
        if shot.duration.before_now(&tick_counter) {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ShotHitEvent {
    pub transform: Transform,
}

// Note: This iterates through all collision_events. We should use one system
// for all such intersections to avoid duplicate work.
pub fn shot_hit_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    shot_query: Query<(Entity, &Transform, &Velocity, &ReadMassProperties, &Shot)>,
    mut health_query: Query<&mut Health>,
    mut momentum_query: Query<(&mut Velocity, &ReadMassProperties), Without<Shot>>,
    mut hit_event_writer: EventWriter<ShotHitEvent>,
) {
    let mut shots_to_despawn: SmallVec<[(Entity, Transform); 10]> = smallvec::SmallVec::new();
    for collision_event in collision_events.iter() {
        let CollisionEvent::Started(e1, e2, _flags) = collision_event else { continue; };
        let (shot_entity, shot_transform, shot_vel, shot_mass, shot, target_entity) =
            if let Ok((e, t, v, m, s)) = shot_query.get(*e1) {
                (e, t.to_owned(), v, m, s, e2)
            } else if let Ok((e, t, v, m, s)) = shot_query.get(*e2) {
                (e, t.to_owned(), v, m, s, e1)
            } else {
                continue;
            };

        shots_to_despawn.push((shot_entity, shot_transform));
        if let Ok(mut health) = health_query.get_mut(*target_entity) {
            health.cur -= shot.damage;
        }
        if let Ok((mut vel, mass)) = momentum_query.get_mut(*target_entity) {
            vel.linvel += shot_vel.linvel * shot_mass.0.mass / mass.0.mass;
        }
    }
    shots_to_despawn.sort_by_key(|(entity, _transform)| *entity);
    shots_to_despawn.dedup_by_key(|(entity, _transform)| *entity);

    for (entity, transform) in shots_to_despawn.into_iter() {
        commands.entity(entity).despawn();
        hit_event_writer.send(ShotHitEvent { transform });
    }
}
