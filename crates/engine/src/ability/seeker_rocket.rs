use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query},
};
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, ExternalForce, LockedAxes, ReadMassProperties,
    Sensor, Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};

use crate::{
    collision::{Colliding, TrackCollisions},
    death_callback::{DeathCallback, ExplosionCallback},
    level::InLevel,
    movement::DesiredMove,
    time::{Tick, TickCounter},
    Health, Kind, Object, Target, To2d, FORWARD, PLAYER_R,
};

use super::{bullet::Bullet, properties::SeekerRocketProps, ABILITY_Y};

#[derive(Component)]
pub struct SeekerRocket {
    pub shooter: Entity,
    pub expiration: Tick,
    pub radius: f32,
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
    let mut rocket_transform = *transform;
    let dir = transform.rotation * FORWARD;
    rocket_transform.translation =
        transform.translation + dir * (PLAYER_R + props.length * 2.0) + ABILITY_Y;

    commands.spawn((
        Object {
            transform: rocket_transform,
            global_transform: GlobalTransform::default(),
            collider: Collider::capsule_z(props.length * 0.5, props.radius),
            mass_props: ColliderMassProperties::Density(1.0),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            velocity: *velocity,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            mass: ReadMassProperties::default(),
            kind: Kind::SeekerRocket,
            in_level: InLevel,
        },
        Health::new(props.health),
        SeekerRocket {
            expiration: tick_counter.at(props.duration),
            shooter,
            radius: props.radius,
            turning_radius: props.turning_radius,
        },
        props.max_speed,
        DeathCallback::Explosion(ExplosionCallback {
            damage: props.damage,
            radius: props.explosion_radius,
        }),
        ExternalForce::default(),
        DesiredMove::default(),
        ActiveEvents::COLLISION_EVENTS,
        TrackCollisions,
        Sensor,
    ));
}

pub fn seeker_rocket_tracking(
    mut query: Query<(&SeekerRocket, &mut Transform, &mut DesiredMove)>,
    target_query: Query<&Target>,
) {
    for (rocket, mut transform, mut desired_move) in query.iter_mut() {
        // Rockets always go forward.
        desired_move.dir = (transform.rotation * FORWARD).to_2d();

        let Ok(target) = target_query.get(rocket.shooter) else {
            continue;
        };
        let target = target.0;

        let facing = transform.forward().to_2d();

        let desired_rotation = facing.angle_between(target - transform.translation.to_2d());
        let rotation = desired_rotation.clamp(-rocket.turning_radius, rocket.turning_radius);

        transform.rotate_y(rotation);
    }
}

pub fn seeker_rocket_collision_system(
    mut rocket_q: Query<(&mut Health, &Colliding), With<SeekerRocket>>,
    // TODO: For now, we explode rockets on contact with anything but a bullet.
    // Let's be smarter about this.
    bullet_q: Query<(), (With<Bullet>, Without<SeekerRocket>)>,
) {
    for (mut health, colliding) in &mut rocket_q {
        let mut die = false;

        for &target in &colliding.targets {
            if bullet_q.get(target).is_err() {
                die = true;
            }
        }

        if die {
            health.die();
        }
    }
}
