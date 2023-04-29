

use bevy::prelude::{Commands, Component, Entity, GlobalTransform, Query, Transform, Vec2, Vec3};
use bevy_rapier3d::prelude::{
    Ccd, Collider, ColliderMassProperties, LockedAxes, ReadMassProperties, Velocity,
};

use crate::{
    physics::G,
    time::{Tick, TickCounter},
    Cooldowns, Energy, Object, PLAYER_R,
};

/// Calculate the initial velocity of a projectile thrown at 45 degrees up, so
/// that it will land at target.
fn calculate_initial_vel(spawn: Vec2, target: Vec2) -> Velocity {
    let dir_in_plane = target - spawn;
    let dist = dir_in_plane.length();

    let v0 = (G * dist).sqrt();

    let z = dist;
    let dir = dir_in_plane.extend(z).normalize();
    let linvel = v0 * dir;

    Velocity {
        linvel,
        angvel: Vec3::ZERO,
    }
}

const FRAG_GRENADE_ENERGY: f32 = 20.0;
const FRAG_GRENADE_COOLDOWN: Tick = Tick(30);
const FRAG_GRENADE_DELAY: Tick = Tick(120);
const FRAG_GRENADE_DAMAGE: f32 = 20.0;
const FRAG_GRENADE_EXP_RADIUS: f32 = 5.0;
const FRAG_GRENADE_R: f32 = 0.30;

#[derive(Component)]
pub struct FragGrenade {
    shooter: Entity,
    duration: Tick,
    damage: f32,
    radius: f32,
    explosion_radius: f32,
}

pub fn frag_grenade(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    energy: &mut Energy,
    tick_counter: &TickCounter,
    transform: &Transform,
    shooter: Entity,
    target: Vec2,
) -> bool {
    // FIXME: use real target
    if cooldowns.frag_grenade.before_now(tick_counter) && energy.cur >= FRAG_GRENADE_ENERGY {
        energy.cur -= FRAG_GRENADE_ENERGY;
        cooldowns.frag_grenade = tick_counter.at(FRAG_GRENADE_COOLDOWN);

        let dir = transform.rotation * Vec3::Y;
        let position = transform.translation + dir * (PLAYER_R + FRAG_GRENADE_R + 0.01);
        let vel = calculate_initial_vel(position.truncate(), target);

        commands.spawn((
            Object {
                transform: Transform::from_translation(position).with_scale(Vec3::new(
                    FRAG_GRENADE_R,
                    FRAG_GRENADE_R,
                    FRAG_GRENADE_R,
                )),
                global_transform: GlobalTransform::default(),
                collider: Collider::ball(FRAG_GRENADE_R),
                mass_props: ColliderMassProperties::Density(1.0),
                body: bevy_rapier3d::prelude::RigidBody::Dynamic,
                velocity: vel,
                locked_axes: LockedAxes::ROTATION_LOCKED,
                mass: ReadMassProperties::default(),
            },
            FragGrenade {
                duration: tick_counter.at(FRAG_GRENADE_DELAY),
                shooter,
                radius: FRAG_GRENADE_R,
                damage: FRAG_GRENADE_DAMAGE,
                explosion_radius: FRAG_GRENADE_EXP_RADIUS,
            },
            Ccd::enabled(),
        ));
        true
    } else {
        false
    }
}

pub fn grenade_land_system(
    mut query: Query<(&FragGrenade, &mut LockedAxes, &mut Transform, &mut Velocity)>,
) {
    for (grenade, mut axes, mut transform, mut velocity) in &mut query {
        if transform.translation.z < grenade.radius && velocity.linvel.z < 0.0 {
            transform.translation.z = grenade.radius;
            *axes |= LockedAxes::TRANSLATION_LOCKED_Z;
            velocity.linvel = Vec3::ZERO;
        }
    }
}
