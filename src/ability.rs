use std::{f32::consts::PI, time::Duration};

use bevy::prelude::{
    Added, Commands, Component, Entity, EventReader, EventWriter, GlobalTransform, Quat, Query,
    Res, Transform, Vec3, With, Without,
};

use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, ColliderMassProperties, CollisionEvent, LockedAxes,
    ReadMassProperties, RigidBody, Sensor, Velocity,
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    status_effect::{StatusEffect, StatusEffects},
    time::{Tick, TickCounter},
    Cooldowns, Energy, Health, Object, PLAYER_R,
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
        just_pressed: bool,
        commands: &mut Commands,
        tick_counter: &TickCounter,
        entity: Entity,
        energy: &mut Energy,
        cooldowns: &mut Cooldowns,
        transform: &Transform,
        velocity: &Velocity,
        status_effects: &mut StatusEffects,
    ) -> bool {
        match self {
            Ability::None => true,
            Ability::HyperSprint => {
                if just_pressed {
                    hyper_sprint(commands, entity, energy, status_effects)
                } else {
                    false
                }
            }
            Ability::Shoot => shoot(
                commands,
                cooldowns,
                energy,
                tick_counter,
                transform,
                velocity,
                entity,
            ),
            Ability::Shotgun => shotgun(
                commands,
                cooldowns,
                energy,
                tick_counter,
                transform,
                velocity,
                entity,
            ),
        }
    }

    pub fn unfire(
        &self,
        commands: &mut Commands,
        entity: Entity,
        status_effects: &mut StatusEffects,
    ) {
        match self {
            Ability::None => (),
            Ability::HyperSprint => {
                hyper_sprint_disable(commands, entity, status_effects);
            }
            Ability::Shoot => (),
            Ability::Shotgun => (),
        }
    }
}

#[derive(Component, Hash)]
pub struct HyperSprinting;
pub const HYPER_SPRINT_FACTOR: f32 = 7.0;
const HYPER_SPRINT_COST: f32 = 2.0;

fn hyper_sprint(
    commands: &mut Commands,
    entity: Entity,
    energy: &Energy,
    status_effects: &mut StatusEffects,
) -> bool {
    if energy.cur >= HYPER_SPRINT_COST {
        commands.entity(entity).insert(HyperSprinting);
        status_effects.effects.insert(StatusEffect::HyperSprinting);
        true
    } else {
        false
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    mut query: Query<(&mut Energy, Entity, &mut StatusEffects), With<HyperSprinting>>,
) {
    for (mut energy, entity, mut status_effects) in &mut query {
        if energy.cur >= HYPER_SPRINT_COST {
            energy.cur -= HYPER_SPRINT_COST;
        } else {
            hyper_sprint_disable(&mut commands, entity, &mut status_effects);
        }
    }
}

fn hyper_sprint_disable(
    commands: &mut Commands,
    entity: Entity,
    status_effects: &mut StatusEffects,
) {
    status_effects.effects.remove(&StatusEffect::HyperSprinting);
    commands.entity(entity).remove::<HyperSprinting>();
}

pub const SHOOT_COOLDOWN: Tick = Tick::new(Duration::from_millis(250));
const SHOT_DURATION: Tick = Tick::new(Duration::from_secs(10));
pub const SHOT_SPEED: f32 = 30.0;
pub const SHOT_R: f32 = 0.15;
pub const ABILITY_Z: f32 = 1.5;
const SHOT_DAMAGE: f32 = 1.0;

const SHOT_ENERGY: f32 = 5.0;

#[derive(Component)]
pub struct Shot {
    shooter: Entity,
    duration: Tick,
    damage: f32,
}

fn shoot(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    energy: &mut Energy,
    tick_counter: &TickCounter,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) -> bool {
    if cooldowns.shoot.before_now(tick_counter) && energy.cur >= SHOT_ENERGY {
        energy.cur -= SHOT_ENERGY;
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

pub const SHOTGUN_COOLDOWN: Tick = Tick::new(Duration::from_millis(250));
const SHOTGUN_DURATION: Tick = Tick::new(Duration::from_secs(10));
pub const SHOTGUN_SPEED: f32 = 30.0;
pub const SHOTGUN_R: f32 = 0.15;
const SHOTGUN_DAMAGE: f32 = 1.0;
const N_PELLETS: usize = 8;
const SPREAD: f32 = PI * 0.125; // Spread angle in radians
const SHOTGUN_ENERGY: f32 = 30.0;

fn shotgun(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    energy: &mut Energy,
    tick_counter: &TickCounter,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) -> bool {
    if cooldowns.shotgun.before_now(tick_counter) && energy.cur >= SHOTGUN_ENERGY {
        cooldowns.shotgun = tick_counter.at(SHOTGUN_COOLDOWN);
        energy.cur -= SHOTGUN_ENERGY;

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
