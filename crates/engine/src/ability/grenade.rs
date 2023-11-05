use std::f32::consts::PI;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventWriter},
    system::{Commands, Query, Res},
};
use bevy_hierarchy::{despawn_with_children_recursive, DespawnRecursiveExt};
use bevy_math::{Vec2, Vec3};
use bevy_rapier3d::prelude::{
    Ccd, Collider, ColliderMassProperties, LockedAxes, RapierContext, ReadMassProperties, Sensor,
    Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};
use nalgebra::ComplexField;

use crate::{
    physics::G,
    time::{Tick, TickCounter},
    Health, Object, Target, PLAYER_R,
};

use super::properties::GrenadeProps;

/// Calculate the initial velocity of a projectile thrown at 45 degrees up, so
/// that it will land at target.
fn calculate_initial_vel(spawn: Vec2, target: Vec2) -> Velocity {
    let dir_in_plane = target - spawn;
    let dist = dir_in_plane.length();

    let phi = PI / 12.0;

    let sin2phi = ComplexField::sin(2.0 * phi);
    let tan = ComplexField::tan(phi);
    let v0 = (dist * G / sin2phi).sqrt();

    let z = dist * tan;
    let dir = dir_in_plane.extend(z).normalize();
    let linvel = v0 * dir;

    Velocity {
        linvel,
        angvel: Vec3::ZERO,
    }
}

#[derive(Clone, Copy)]
pub enum GrenadeKind {
    Frag,
    Heal,
}

#[derive(Component)]
pub struct Grenade {
    // TODO: Use this field
    #[allow(dead_code)]
    shooter: Entity,
    expiration: Tick,
    damage: f32,
    radius: f32,
    explosion_radius: f32,
    pub kind: GrenadeKind,
}

pub fn grenade(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &GrenadeProps,
    transform: &Transform,
    shooter: Entity,
    target: &Target,
) {
    let dir = transform.rotation * Vec3::Y;
    let position = transform.translation + dir * (PLAYER_R + props.radius + 0.01);
    let vel = calculate_initial_vel(position.truncate(), target.0);

    commands.spawn((
        Object {
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            collider: Collider::ball(props.radius),
            mass_props: ColliderMassProperties::Density(1.0),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            velocity: vel,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            mass: ReadMassProperties::default(),
        },
        Grenade {
            expiration: tick_counter.at(props.delay),
            shooter,
            radius: props.radius,
            damage: props.damage,
            explosion_radius: props.explosion_radius,
            kind: GrenadeKind::Frag,
        },
        Ccd::enabled(),
    ));
}

#[derive(Event)]
pub struct GrenadeLandEvent {
    pub entity: Entity,
}

pub fn grenade_land_system(
    mut query: Query<(
        Entity,
        &Grenade,
        &mut LockedAxes,
        &mut Transform,
        &mut Velocity,
    )>,
    mut event_writer: EventWriter<GrenadeLandEvent>,
) {
    for (entity, grenade, mut axes, mut transform, mut velocity) in &mut query {
        if transform.translation.z < grenade.radius && velocity.linvel.z < 0.0 {
            transform.translation.z = grenade.radius;
            *axes |= LockedAxes::TRANSLATION_LOCKED_Z;
            velocity.linvel = Vec3::ZERO;
            event_writer.send(GrenadeLandEvent { entity });
        }
    }
}

#[derive(Component)]
pub struct Explosion {
    pub damage: f32,
    pub kind: GrenadeKind,
}

// Explosions only last one frame.
pub fn explosion_despawn_system(
    mut commands: Commands,
    query: Query<(Entity, &Explosion)>,
    mut health_query: Query<&mut Health>,
    rapier: Res<RapierContext>,
) {
    for (entity, explosion) in &query {
        for (e1, e2, intersecting) in rapier.intersections_with(entity) {
            if intersecting {
                let other = if e1 == entity { e2 } else { e1 };
                if let Ok(mut health) = health_query.get_mut(other) {
                    health.take(explosion.damage);
                }
            }
        }
        // FIXME
        // commands.entity(entity).despawn_recursive();
    }
}

pub fn grenade_explode_system(
    mut commands: Commands,
    query: Query<(Entity, &Grenade, &Transform)>,
    tick_counter: Res<TickCounter>,
) {
    for (entity, grenade, transform) in &query {
        if grenade.expiration.before_now(&tick_counter) {
            commands.entity(entity).despawn_recursive();
            commands.spawn((
                Object {
                    transform: *transform,
                    collider: Collider::ball(grenade.explosion_radius),
                    body: bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
                    ..Default::default()
                },
                Sensor,
                Explosion {
                    damage: grenade.damage,
                    kind: grenade.kind,
                },
            ));
        }
    }
}
