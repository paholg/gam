use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    query::{Added, With, Without},
    system::{Commands, Query, Res},
};
use bevy_math::{Quat, Vec2, Vec3};
use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, ColliderMassProperties, CollisionEvent, LockedAxes,
    ReadMassProperties, RigidBody, Sensor, Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use strum::{Display, EnumIter};
use tracing::warn;

use crate::{
    status_effect::{StatusEffect, StatusEffects},
    time::{Tick, TickCounter},
    Cooldowns, Energy, Health, Object, Shootable, PLAYER_R,
};

use self::{
    grenade::grenade,
    properties::{AbilityProps, HyperSprintProps, ShootProps, ShotgunProps},
};

pub mod grenade;
pub mod properties;

pub const ABILITY_Z: f32 = 1.5;

#[derive(
    Debug, Copy, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash, EnumIter, Display,
)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Shoot,
    Shotgun,
    FragGrenade,
    HealGrenade,
}

impl Ability {
    #[allow(clippy::too_many_arguments)]
    pub fn fire(
        &self,
        just_pressed: bool,
        commands: &mut Commands,
        tick_counter: &TickCounter,
        props: &AbilityProps,
        entity: Entity,
        energy: &mut Energy,
        cooldowns: &mut Cooldowns,
        transform: &Transform,
        velocity: &Velocity,
        status_effects: &mut StatusEffects,
        target: Vec2,
    ) -> bool {
        let cooldown = match cooldowns.map.get_mut(self) {
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
            Ability::HyperSprint => {
                if just_pressed {
                    hyper_sprint(
                        commands,
                        &props.hyper_sprint,
                        entity,
                        energy,
                        status_effects,
                    );
                }
            }
            Ability::Shoot => shoot(
                commands,
                tick_counter,
                &props.shoot,
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
            Ability::Shoot => (),
            Ability::Shotgun => (),
            Ability::FragGrenade => (),
            Ability::HealGrenade => (),
        }
    }
}

#[derive(Component, Hash)]
pub struct HyperSprinting;

fn hyper_sprint(
    commands: &mut Commands,
    props: &HyperSprintProps,
    entity: Entity,
    energy: &Energy,
    status_effects: &mut StatusEffects,
) -> bool {
    if energy.cur >= props.cost {
        commands.entity(entity).insert(HyperSprinting);
        status_effects.effects.insert(StatusEffect::HyperSprinting);
        true
    } else {
        false
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    props: Res<AbilityProps>,
    mut query: Query<(&mut Energy, Entity, &mut StatusEffects), With<HyperSprinting>>,
) {
    for (mut energy, entity, mut status_effects) in &mut query {
        if !energy.try_use(props.hyper_sprint.cost) {
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

#[derive(Component)]
pub struct Shot {
    shooter: Entity,
    duration: Tick,
    damage: f32,
}

fn shoot(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &ShootProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) {
    let dir = transform.rotation * Vec3::Y;
    let position =
        transform.translation + dir * (PLAYER_R + props.radius + 0.01) + ABILITY_Z * Vec3::Z;
    let vel = dir * props.speed + velocity.linvel;
    commands.spawn((
        Object {
            transform: Transform::from_translation(position).with_scale(Vec3::new(
                props.radius,
                props.radius,
                props.radius,
            )),
            global_transform: GlobalTransform::default(),
            collider: Collider::ball(props.radius),
            mass_props: ColliderMassProperties::Density(100.0),
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
            duration: tick_counter.at(props.duration),
            shooter,
            damage: props.damage,
        },
        Ccd::enabled(),
        ActiveEvents::COLLISION_EVENTS,
    ));
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
            transform.translation + dir * (PLAYER_R + props.radius + 0.01) + ABILITY_Z * Vec3::Z;
        let vel = dir * props.speed + velocity.linvel;
        commands.spawn((
            Object {
                transform: Transform::from_translation(position).with_scale(Vec3::new(
                    props.radius,
                    props.radius,
                    props.radius,
                )),
                global_transform: GlobalTransform::default(),
                collider: Collider::ball(props.radius),
                mass_props: ColliderMassProperties::Density(100.0),
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
                duration: tick_counter.at(props.duration),
                shooter,
                damage: props.damage,
            },
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

#[derive(Event)]
pub struct ShotHitEvent {
    pub transform: Transform,
}

// Note: This iterates through all collision_events. We should use one system
// for all such intersections to avoid duplicate work.
pub fn shot_hit_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    shot_query: Query<(Entity, &Transform, &Velocity, &ReadMassProperties, &Shot)>,
    // explosion_query: Query<&Explosion>,
    mut health_query: Query<&mut Health>,
    mut momentum_query: Query<
        (&mut Velocity, &ReadMassProperties),
        (With<Shootable>, Without<Shot>),
    >,
    mut hit_event_writer: EventWriter<ShotHitEvent>,
) {
    let mut shots_to_despawn: SmallVec<[(Entity, Transform); 10]> = smallvec::SmallVec::new();
    for collision_event in collision_events.iter() {
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
