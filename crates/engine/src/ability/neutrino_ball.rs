use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::Without,
    system::{Commands, Query},
};
use bevy_hierarchy::BuildChildren;
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, ExternalForce, LockedAxes, ReadMassProperties, Sensor, Velocity,
};
use bevy_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};

use crate::{
    collision::TrackCollisions,
    level::InLevel,
    status_effect::{StatusBundle, TimeDilation},
    time::Dur,
    AbilityOffset, FootOffset, Health, Kind, MassBundle, Object, Shootable, FORWARD, PLAYER_R,
};

use super::properties::NeutrinoBallProps;

#[derive(Component)]
pub struct NeutrinoBall {
    pub accel_numerator: f32,
    pub radius: f32,
    pub effect_radius: f32,
    pub activates_in: Dur,
}

pub fn neutrino_ball(
    commands: &mut Commands,
    props: &NeutrinoBallProps,
    transform: &Transform,
    velocity: &Velocity,
    ability_offset: &AbilityOffset,
) {
    let mut ball_transform = *transform;
    let dir = transform.rotation * FORWARD;

    ball_transform.translation =
        transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ability_offset.to_vec();

    let ball_velocity = velocity.linvel + dir * props.speed;

    commands.spawn((
        Object {
            transform: ball_transform.into(),
            collider: Collider::ball(props.radius),
            foot_offset: (-props.radius).into(),
            mass: MassBundle::new(props.mass()),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: Velocity::linear(ball_velocity),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            kind: Kind::NeutrinoBall,
            in_level: InLevel,
            statuses: StatusBundle::default(),
        },
        NeutrinoBall {
            accel_numerator: props.accel_numerator(),
            radius: props.radius,
            effect_radius: props.effect_radius,
            activates_in: props.activation_delay,
        },
        Health::new_with_delay(0.0, props.duration),
        Shootable,
    ));
}

#[derive(Component)]
pub struct NeutrinoBallGravityFieldSpawned;

#[derive(Component)]
pub struct NeutrinoBallGravityField {
    accel_numerator: f32,
}

pub fn activation_system(
    mut commands: Commands,
    mut neutrino_q: Query<
        (Entity, &mut NeutrinoBall, &FootOffset, &TimeDilation),
        Without<NeutrinoBallGravityFieldSpawned>,
    >,
) {
    for (entity, mut ball, foot_offset, time_dilation) in &mut neutrino_q {
        if ball.activates_in.tick(time_dilation) {
            commands
                .entity(entity)
                .insert(NeutrinoBallGravityFieldSpawned)
                .with_children(|builder| {
                    builder.spawn((
                        NeutrinoBallGravityField {
                            accel_numerator: ball.accel_numerator,
                        },
                        *foot_offset,
                        TransformBundle::default(),
                        ActiveEvents::COLLISION_EVENTS,
                        TrackCollisions::default(),
                        Collider::ball(ball.effect_radius),
                        Sensor,
                    ));
                });
        }
    }
}

pub fn collision_system(
    neutrino_q: Query<(
        &NeutrinoBallGravityField,
        &GlobalTransform,
        &TrackCollisions,
    )>,
    mut target_q: Query<(
        &mut ExternalForce,
        &Transform,
        &ReadMassProperties,
        &TimeDilation,
    )>,
) {
    for (field, global_transform, colliding) in &neutrino_q {
        for &target in &colliding.targets {
            let Ok((mut force, target_transform, mass, dilation)) = target_q.get_mut(target) else {
                // TODO: Exclude `Floor` before this.
                // tracing::warn!(?target, "Neutrino ball hit target, but not in query");
                continue;
            };
            let translation = global_transform.translation();
            let d2 = translation.distance_squared(target_transform.translation);

            let a = field.accel_numerator / d2;
            let f = mass.mass * a;

            let mut dir = (translation - target_transform.translation).normalize();
            // Let's keep it from letting you fly up, for now.
            dir.y = 0.0;

            force.force += f * dilation.factor() * dir;
        }
    }
}
