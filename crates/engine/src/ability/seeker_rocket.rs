use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, ExternalImpulse, LockedAxes,
    ReadMassProperties, Sensor, Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};

use crate::{
    time::{Tick, TickCounter},
    Health, Object, Target, DAMPING, PLAYER_R,
};

use super::{
    explosion::{Explosion, ExplosionKind},
    properties::SeekerRocketProps,
    ABILITY_Z,
};

#[derive(Component)]
pub struct SeekerRocket {
    pub shooter: Entity,
    pub expiration: Tick,
    pub damage: f32,
    pub radius: f32,
    pub explosion_radius: f32,
    pub max_impulse: f32,
    pub turning_radius: f32,
}

pub fn seeker_rocket(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &SeekerRocketProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
) {
    let mut rocket_transform = transform.clone();
    let dir = transform.rotation * Vec3::Y;
    rocket_transform.translation =
        transform.translation + dir * (PLAYER_R + props.length * 2.0) + ABILITY_Z * Vec3::Z;

    commands.spawn((
        Object {
            transform: rocket_transform,
            global_transform: GlobalTransform::default(),
            collider: Collider::capsule_y(props.length * 0.5, props.radius),
            mass_props: ColliderMassProperties::Density(1.0),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            velocity: *velocity,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            mass: ReadMassProperties::default(),
        },
        Health::new(props.health),
        SeekerRocket {
            expiration: tick_counter.at(props.duration),
            shooter,
            radius: props.radius,
            damage: props.damage,
            explosion_radius: props.explosion_radius,
            max_impulse: props.max_impulse,
            turning_radius: props.turning_radius,
        },
        DAMPING,
        ExternalImpulse::default(),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    ));
}

pub fn seeker_rocket_tracking(
    mut query: Query<(&SeekerRocket, &mut Transform, &mut ExternalImpulse)>,
    target_query: Query<&Target>,
) {
    for (rocket, mut transform, mut impulse) in query.iter_mut() {
        // Set impulse whether or not we have a target.
        impulse.impulse = transform.rotation * Vec3::Y * rocket.max_impulse;

        let Ok(target) = target_query.get(rocket.shooter) else {
            continue;
        };
        let target = target.0;

        // TODO: Why is this up and not forward?
        let facing = transform.up().truncate();

        let desired_rotation = facing.angle_between(target - transform.translation.truncate());
        let rotation = desired_rotation.clamp(-rocket.turning_radius, rocket.turning_radius);

        transform.rotate_z(rotation);
    }
}

pub fn explode(
    commands: &mut Commands,
    entity: Entity,
    rocket: &SeekerRocket,
    transform: &Transform,
) {
    commands.entity(entity).despawn_recursive();
    commands.spawn((
        Object {
            transform: *transform,
            collider: Collider::ball(rocket.explosion_radius),
            body: bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
            ..Default::default()
        },
        Sensor,
        Explosion {
            damage: rocket.damage,
            kind: ExplosionKind::SeekerRocket,
        },
    ));
}
