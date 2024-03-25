use std::f32::consts::PI;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query},
};
use bevy_rapier3d::prelude::{Collider, ExternalForce, LockedAxes, Sensor, Velocity};
use bevy_transform::components::Transform;

use super::properties::ExplosionProps;
use crate::{
    collision::{TrackCollisionBundle, TrackCollisions},
    death_callback::{DeathCallback, ExplosionCallback, ExplosionKind},
    level::InLevel,
    movement::{DesiredMove, MaxSpeed},
    status_effect::StatusProps,
    time::{Dur, TIMESTEP},
    AbilityOffset, Energy, Health, Kind, MassBundle, Object, Shootable, Target, To2d, FORWARD,
    PLAYER_R,
};

#[derive(Debug)]
pub struct SeekerRocketProps {
    pub cost: f32,
    pub cooldown: Dur,
    /// Max turn per tick, in radians.
    pub turning_radius: f32,
    pub capsule_radius: f32,
    pub capsule_length: f32,
    pub health: f32,
    pub max_speed: MaxSpeed,
    /// How much energy the rocket has.
    pub energy: f32,
    /// How much energy the rocket spends every frame to move.
    pub energy_cost: f32,
    pub explosion: ExplosionProps,
    pub mass: f32,
}

impl Default for SeekerRocketProps {
    fn default() -> Self {
        Self {
            cost: 30.0,
            cooldown: Dur::new(30),
            turning_radius: PI * 0.03,
            capsule_radius: 0.05,
            capsule_length: 0.14,
            health: 3.0,
            max_speed: MaxSpeed {
                accel: 1800.0,
                speed: 8.0,
            },
            energy: 10.0,
            energy_cost: 0.2,
            explosion: ExplosionProps {
                min_radius: 0.2,
                max_radius: 1.2,
                duration: Dur::new(15),
                damage: 0.6,
                force: 300.0,
                kind: ExplosionKind::SeekerRocket,
            },
            mass: 2.0,
        }
    }
}

#[derive(Component)]
pub struct SeekerRocket {
    pub shooter: Entity,
    pub radius: f32,
    pub turning_radius: f32,
    pub energy_cost: f32,
}

pub fn seeker_rocket(
    commands: &mut Commands,
    props: &SeekerRocketProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
    ability_offset: &AbilityOffset,
) {
    let mut rocket_transform = *transform;
    let dir = transform.rotation * FORWARD;
    // TODO: If the rocket spawns inside a wall, no one will be hurt by its
    // explosion.
    rocket_transform.translation = transform.translation
        + dir * (PLAYER_R + props.capsule_length * 2.0)
        + ability_offset.to_vec();

    commands.spawn((
        Object {
            transform: rocket_transform.into(),
            collider: Collider::capsule_z(props.capsule_length * 0.5, props.capsule_radius),
            foot_offset: (-props.capsule_radius).into(),
            mass: MassBundle::new(props.mass),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: *velocity,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            kind: Kind::SeekerRocket,
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::on(),
        },
        Health::new(props.health),
        Energy::new(props.energy, 0.0),
        SeekerRocket {
            shooter,
            radius: props.capsule_radius,
            turning_radius: props.turning_radius,
            energy_cost: props.energy_cost,
        },
        Shootable,
        props.max_speed,
        DeathCallback::Explosion(ExplosionCallback {
            props: props.explosion,
        }),
        DesiredMove {
            can_fly: true,
            ..Default::default()
        },
        Sensor,
    ));
}

pub fn tracking_system(
    mut query: Query<(
        &SeekerRocket,
        &mut Transform,
        &mut DesiredMove,
        &mut Energy,
        &mut LockedAxes,
    )>,
    target_query: Query<&Target>,
) {
    for (rocket, mut transform, mut desired_move, mut energy, mut locked_axes) in query.iter_mut() {
        if energy.try_use(rocket.energy_cost) {
            let Ok(target) = target_query.get(rocket.shooter) else {
                continue;
            };
            let target = target.0;

            let facing = transform.forward().to_2d();

            let desired_rotation = facing.angle_between(target - transform.translation.to_2d());
            let rotation = desired_rotation.clamp(-rocket.turning_radius, rocket.turning_radius);

            transform.rotate_y(rotation);

            // Rockets always go forward.
            desired_move.dir = (transform.rotation * FORWARD).to_2d();
        } else {
            // Unlock y translation, so it can fall.
            *locked_axes = LockedAxes::ROTATION_LOCKED;
        }
    }
}

pub fn collision_system(
    mut rocket_q: Query<
        (&mut Health, &TrackCollisions, &Velocity, &mut Transform),
        With<SeekerRocket>,
    >,
    shootable_q: Query<(), With<Shootable>>,
) {
    for (mut health, colliding, velocity, mut transform) in &mut rocket_q {
        let should_live = colliding.targets.is_empty()
            || colliding
                .targets
                .iter()
                .all(|&t| shootable_q.get(t).is_err());

        if !should_live {
            health.die();
            // If a rocket hits a wall, it will be inside it when it explodes,
            // damaging no one. So, we just move back a frame.
            transform.translation -= velocity.linvel * TIMESTEP;
        }
    }
}
