use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query},
};
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, ExternalForce, LockedAxes, ReadMassProperties,
    Sensor, Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};

use crate::{
    collision::TrackCollisions,
    death_callback::{DeathCallback, ExplosionCallback},
    level::InLevel,
    movement::DesiredMove,
    time::TIMESTEP,
    AbilityOffset, Energy, Health, Kind, Object, Shootable, Target, To2d, FORWARD, PLAYER_R,
};

use super::properties::SeekerRocketProps;

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
            transform: rocket_transform,
            global_transform: GlobalTransform::default(),
            collider: Collider::capsule_z(props.capsule_length * 0.5, props.capsule_radius),
            foot_offset: (-props.capsule_radius).into(),
            mass_props: ColliderMassProperties::Density(1.0),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: *velocity,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            mass: ReadMassProperties::default(),
            kind: Kind::SeekerRocket,
            in_level: InLevel,
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
        ActiveEvents::COLLISION_EVENTS,
        TrackCollisions::default(),
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
