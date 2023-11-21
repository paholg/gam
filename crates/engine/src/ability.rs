use std::ops::Index;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    query::{Added, With, Without},
    system::{Commands, Query, Res},
};
use bevy_math::{Quat, Vec3};
use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, ColliderMassProperties, CollisionEvent, LockedAxes,
    ReadMassProperties, RigidBody, Sensor, Velocity,
};
use bevy_reflect::Reflect;
use bevy_transform::components::{GlobalTransform, Transform};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use strum::{Display, EnumIter};
use tracing::warn;

use crate::{
    status_effect::{StatusEffect, StatusEffects},
    time::{Tick, TickCounter},
    Cooldowns, Energy, Health, Object, Shootable, Target, PLAYER_R,
};

use self::{
    grenade::grenade,
    properties::{AbilityProps, GunProps, ShotgunProps},
    seeker_rocket::{seeker_rocket, SeekerRocket},
};

pub mod explosion;
pub mod grenade;
pub mod properties;
pub mod seeker_rocket;

pub const ABILITY_Z: f32 = 1.5;
pub const PLAYER_ABILITY_COUNT: usize = 5;

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    EnumIter,
    Display,
    Reflect,
    Hash,
)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Gun,
    Shotgun,
    FragGrenade,
    HealGrenade,
    SeekerRocket,
}

#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct Abilities {
    inner: Vec<Ability>,
}

impl Abilities {
    pub fn new(abilities: Vec<Ability>) -> Self {
        Self { inner: abilities }
    }

    pub fn iter(&self) -> impl Iterator<Item = Ability> + '_ {
        self.inner.iter().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Ability> {
        self.inner.iter_mut()
    }
}

impl Index<usize> for Abilities {
    type Output = Ability;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl Ability {
    #[allow(clippy::too_many_arguments)]
    pub fn fire(
        &self,
        commands: &mut Commands,
        tick_counter: &TickCounter,
        props: &AbilityProps,
        entity: Entity,
        energy: &mut Energy,
        cooldowns: &mut Cooldowns,
        transform: &Transform,
        velocity: &Velocity,
        status_effects: &mut StatusEffects,
        target: &Target,
    ) -> bool {
        let cooldown = match cooldowns.get_mut(self) {
            Some(cd) => cd,
            None => {
                warn!("Tried to use an ability that we don't have a cooldown for");
                return false;
            }
        };

        if cooldown.before_now(tick_counter) && energy.try_use(props.cost(self)) {
            *cooldown = tick_counter.at(props.cooldown(self));
        } else {
            return false;
        }

        match self {
            Ability::None => (),
            Ability::HyperSprint => hyper_sprint(commands, entity, status_effects),
            Ability::Gun => gun(
                commands,
                tick_counter,
                &props.gun,
                transform,
                velocity,
                entity,
            ),
            Ability::Shotgun => shotgun(
                commands,
                tick_counter,
                &props.shotgun,
                transform,
                velocity,
                entity,
            ),
            Ability::FragGrenade => grenade(
                commands,
                tick_counter,
                &props.frag_grenade,
                transform,
                entity,
                target,
            ),
            Ability::HealGrenade => grenade(
                commands,
                tick_counter,
                &props.heal_grenade,
                transform,
                entity,
                target,
            ),
            Ability::SeekerRocket => seeker_rocket(
                commands,
                tick_counter,
                &props.seeker_rocket,
                transform,
                velocity,
                entity,
            ),
        }
        true
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
            Ability::Gun => (),
            Ability::Shotgun => (),
            Ability::FragGrenade => (),
            Ability::HealGrenade => (),
            Ability::SeekerRocket => (),
        }
    }
}

#[derive(Component, Hash)]
pub struct HyperSprinting;

fn hyper_sprint(commands: &mut Commands, entity: Entity, status_effects: &mut StatusEffects) {
    commands.entity(entity).insert(HyperSprinting);
    status_effects.effects.insert(StatusEffect::HyperSprinting);
}

fn hyper_sprint_disable(
    commands: &mut Commands,
    entity: Entity,
    status_effects: &mut StatusEffects,
) {
    status_effects.effects.remove(&StatusEffect::HyperSprinting);
    commands.entity(entity).remove::<HyperSprinting>();
}

#[derive(Component)]
pub struct Shot {
    shooter: Entity,
    duration: Tick,
    damage: f32,
}

fn gun(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &GunProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) {
    let dir = transform.rotation * Vec3::Y;
    let position =
        transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ABILITY_Z * Vec3::Z;
    let velocity = dir * props.speed + velocity.linvel;
    Bullet {
        position,
        velocity,
        radius: props.radius,
        density: props.density,
        shot: Shot {
            shooter,
            duration: tick_counter.at(props.duration),
            damage: props.damage,
        },
    }
    .spawn(commands);
}

fn shotgun(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &ShotgunProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) {
    for i in 0..props.n_pellets {
        let idx = i as f32;
        let n_pellets = props.n_pellets as f32;
        let relative_angle = (n_pellets * 0.5 - idx) / n_pellets * props.spread;
        let relative_angle = Quat::from_rotation_z(relative_angle);
        let dir = (transform.rotation * relative_angle) * Vec3::Y;
        let position =
            transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ABILITY_Z * Vec3::Z;
        let velocity = dir * props.speed + velocity.linvel;
        Bullet {
            position,
            velocity,
            radius: props.radius,
            density: props.density,
            shot: Shot {
                shooter,
                duration: tick_counter.at(props.duration),
                damage: props.damage,
            },
        }
        .spawn(commands);
    }
}

struct Bullet {
    pub velocity: Vec3,
    pub position: Vec3,
    pub radius: f32,
    pub density: f32,
    pub shot: Shot,
}

impl Bullet {
    fn spawn(self, commands: &mut Commands) {
        commands.spawn((
            Object {
                transform: Transform::from_translation(self.position).with_scale(Vec3::new(
                    self.radius,
                    self.radius,
                    self.radius,
                )),
                global_transform: GlobalTransform::default(),
                collider: Collider::ball(self.radius),
                mass_props: ColliderMassProperties::Density(self.density),
                body: RigidBody::Dynamic,
                velocity: Velocity {
                    linvel: self.velocity,
                    angvel: Vec3::ZERO,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                mass: ReadMassProperties::default(),
            },
            Sensor,
            self.shot,
            Ccd::enabled(),
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
}

pub fn shot_kickback_system(
    shot_query: Query<(&Velocity, &ReadMassProperties, &Shot), Added<Shot>>,
    mut momentum_query: Query<(&mut Velocity, &ReadMassProperties), Without<Shot>>,
) {
    for (v, m, shot) in shot_query.iter() {
        let Ok((mut shooter_v, shooter_m)) = momentum_query.get_mut(shot.shooter) else {
            continue;
        };
        shooter_v.linvel -= v.linvel * m.get().mass / shooter_m.get().mass;
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

#[derive(Event, Debug)]
pub struct ShotHitEvent {
    pub transform: Transform,
}

// Note: This iterates through all collision_events. We should use one system
// for all such intersections to avoid duplicate work.
pub fn shot_hit_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    shot_query: Query<(Entity, &Transform, &Velocity, &ReadMassProperties, &Shot)>,
    rocket_query: Query<(Entity, &SeekerRocket, &Transform)>,
    mut health_query: Query<&mut Health>,
    mut momentum_query: Query<
        (&mut Velocity, &ReadMassProperties),
        (With<Shootable>, Without<Shot>),
    >,
    mut hit_event_writer: EventWriter<ShotHitEvent>,
) {
    let mut shots_to_despawn: SmallVec<[(Entity, Transform); 10]> = smallvec::SmallVec::new();
    for collision_event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _flags) = collision_event else {
            continue;
        };
        let e1 = *e1;
        let e2 = *e2;

        let (shot_entity, shot_transform, shot_vel, shot_mass, shot, target_entity) =
            if let Ok((e, t, v, m, s)) = shot_query.get(e1) {
                (e, t.to_owned(), v, m, s, e2)
            } else if let Ok((e, t, v, m, s)) = shot_query.get(e2) {
                (e, t.to_owned(), v, m, s, e1)
            } else {
                // FIXME: THIS IS HIDEOUS
                //
                // We are handling the rocket in the `else` of the `let _ = if` branch for shots.
                // Adding any more abilities just means deeper hell.
                let (rocket_entity, rocket, transform) = if let Ok((e, r, t)) = rocket_query.get(e1)
                {
                    (e, r, t)
                } else if let Ok((e, r, t)) = rocket_query.get(e2) {
                    (e, r, t)
                } else {
                    continue;
                };

                seeker_rocket::explode(&mut commands, rocket_entity, rocket, transform);

                continue;
            };

        shots_to_despawn.push((shot_entity, shot_transform));
        if let Ok(mut health) = health_query.get_mut(target_entity) {
            health.take(shot.damage);
        }
        if let Ok((mut vel, mass)) = momentum_query.get_mut(target_entity) {
            vel.linvel += shot_vel.linvel * shot_mass.get().mass / mass.get().mass;
        }
    }
    shots_to_despawn.sort_by_key(|(entity, _transform)| *entity);
    shots_to_despawn.dedup_by_key(|(entity, _transform)| *entity);

    for (entity, transform) in shots_to_despawn.into_iter() {
        commands.entity(entity).despawn();
        hit_event_writer.send(ShotHitEvent { transform });
    }
}
