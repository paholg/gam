use std::f32::consts::PI;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Query, Res},
};

use bevy_math::Vec3;
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Friction, LockedAxes, ReadMassProperties, Restitution,
    Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};

use crate::{
    death_callback::{DeathCallback, ExplosionCallback},
    level::InLevel,
    physics::G,
    time::{Tick, TickCounter},
    AbilityOffset, Health, Kind, Libm, Object, Target, To2d, To3d, FORWARD, PLAYER_R,
};

use super::properties::GrenadeProps;

/// Calculate the initial velocity of a projectile thrown at 45 degrees up, so
/// that it will land at target.
fn calculate_initial_vel(spawn: Vec3, target: Vec3) -> Velocity {
    let dir_in_plane = target.to_2d() - spawn.to_2d();
    let height_delta = target.y - spawn.y;
    let dist_in_plane = dir_in_plane.length();

    // TODO: These can all be constants at some point. Or generated with a proc-
    // macro or build script.
    // Or maybe we'll make "throw angle" customizable.
    let phi = PI / 12.0;
    let cos_phi = Libm::cos(phi);
    let cos_sq_phi = cos_phi * cos_phi;
    let tan_phi = Libm::tan(phi);

    let v0_sq = dist_in_plane * dist_in_plane * G
        / (2.0 * cos_sq_phi * (dist_in_plane * tan_phi - height_delta));
    let v0 = Libm::sqrt(v0_sq);

    let dir = dir_in_plane.to_3d(dist_in_plane * tan_phi).normalize();
    let linvel = v0 * dir;

    Velocity {
        linvel,
        angvel: Vec3::ZERO,
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GrenadeKind {
    Frag,
    Heal,
}

impl From<GrenadeKind> for Kind {
    fn from(value: GrenadeKind) -> Self {
        match value {
            GrenadeKind::Frag => Kind::FragGrenade,
            GrenadeKind::Heal => Kind::HealGrenade,
        }
    }
}

#[derive(Component)]
pub struct Grenade {
    // TODO: Use this field
    #[allow(dead_code)]
    shooter: Entity,
    expiration: Tick,
    pub kind: GrenadeKind,
}

pub fn grenade(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &GrenadeProps,
    transform: &Transform,
    shooter: Entity,
    target: &Target,
    ability_offset: &AbilityOffset,
) {
    let dir = transform.rotation * FORWARD;
    let position =
        transform.translation + dir * (PLAYER_R + props.radius + 0.01) + ability_offset.to_vec();
    let vel = calculate_initial_vel(position, target.0.to_3d(props.radius));

    commands.spawn((
        Object {
            transform: Transform::from_translation(position),
            global_transform: GlobalTransform::default(),
            collider: Collider::ball(props.radius),
            foot_offset: (-props.radius).into(),
            mass_props: ColliderMassProperties::Density(1.0),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            velocity: vel,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            mass: ReadMassProperties::default(),
            kind: props.kind.into(),
            in_level: InLevel,
        },
        Grenade {
            expiration: tick_counter.at(props.delay),
            shooter,
            kind: props.kind,
        },
        Friction {
            coefficient: 100.0,
            ..Default::default()
        },
        Restitution {
            coefficient: 0.0,
            ..Default::default()
        },
        DeathCallback::Explosion(ExplosionCallback {
            damage: props.damage,
            radius: props.explosion_radius,
        }),
        Health::new(props.health),
    ));
}

pub fn grenade_explode_system(
    mut query: Query<(&Grenade, &mut Health)>,
    tick_counter: Res<TickCounter>,
) {
    for (grenade, mut health) in &mut query {
        if grenade.expiration.before_now(&tick_counter) {
            health.die();
        }
    }
}
