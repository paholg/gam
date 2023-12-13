use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query},
};
use bevy_rapier3d::prelude::{
    Collider, ExternalForce, LockedAxes, ReadMassProperties, Sensor, Velocity,
};
use bevy_transform::components::Transform;

use crate::{
    collision::{TrackCollisionBundle, TrackCollisions},
    level::InLevel,
    status_effect::{StatusProps, TimeDilation},
    time::Dur,
    FootOffset, Health, Kind, Libm, MassBundle, Object, FORWARD, PLAYER_R,
};

use super::properties::NeutrinoBallProps;

#[derive(Component)]
pub struct NeutrinoBall {
    pub accel_numerator: f32,
    pub surface_a: f32,
    pub radius: f32,
    pub effect_radius: f32,
    pub activates_in: Dur,
}

pub fn neutrino_ball(
    commands: &mut Commands,
    props: &NeutrinoBallProps,
    transform: &Transform,
    velocity: &Velocity,
    foot_offset: &FootOffset,
) {
    let mut ball_transform = *transform;
    let dir = transform.rotation * FORWARD;

    // Let's spawn this one at the caster's feet.
    ball_transform.translation =
        transform.translation + dir * (PLAYER_R + props.radius) + foot_offset.to_vec();

    let ball_velocity = velocity.linvel + dir * props.speed;

    commands.spawn((
        Object {
            transform: ball_transform.into(),
            collider: Collider::ball(props.effect_radius),
            foot_offset: (0.0).into(),
            mass: MassBundle::new(props.mass()),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: Velocity::linear(ball_velocity),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            kind: Kind::NeutrinoBall,
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::on(),
        },
        NeutrinoBall {
            accel_numerator: props.accel_numerator(),
            surface_a: props.surface_a,
            radius: props.radius,
            effect_radius: props.effect_radius,
            activates_in: props.activation_delay,
        },
        Health::new_with_delay(0.0, props.duration),
        Sensor,
    ));
}

#[derive(Component)]
pub struct NeutrinoBallActivated;

pub fn activation_system(
    mut commands: Commands,
    mut neutrino_q: Query<
        (Entity, &mut NeutrinoBall, &TimeDilation),
        Without<NeutrinoBallActivated>,
    >,
) {
    for (entity, mut ball, time_dilation) in &mut neutrino_q {
        if ball.activates_in.tick(time_dilation) {
            commands.entity(entity).insert(NeutrinoBallActivated);
        }
    }
}

pub fn collision_system(
    neutrino_q: Query<(&NeutrinoBall, &Transform, &TrackCollisions), With<NeutrinoBallActivated>>,
    mut target_q: Query<(
        &mut ExternalForce,
        &Transform,
        &ReadMassProperties,
        &TimeDilation,
    )>,
) {
    for (ball, transform, colliding) in &neutrino_q {
        for &target in &colliding.targets {
            let Ok((mut force, target_transform, mass, dilation)) = target_q.get_mut(target) else {
                // TODO: Exclude `Floor` before this. Or maybe it doesn't
                // matter.
                // tracing::warn!(?target, "Neutrino ball hit target, but not in query");
                continue;
            };
            let translation = transform.translation;
            let d2 = translation.distance_squared(target_transform.translation);

            let a = if d2 < ball.radius * ball.radius {
                // Per Gauss' law, if inside the ball's radius, gravity linearly
                // decreases as we approach the center.
                let d = Libm::sqrt(d2);
                let factor = d / ball.radius;
                factor * ball.surface_a
            } else {
                ball.accel_numerator / d2
            };
            let f = mass.mass * a;

            let mut dir = (translation - target_transform.translation).normalize();
            // Let's keep it from letting you fly up, for now.
            dir.y = 0.0;

            force.force += f * dilation.factor() * dir;
        }
    }
}
