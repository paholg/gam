use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::Without,
    system::{Commands, Query, Res},
};
use bevy_hierarchy::BuildChildren;
use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, ExternalForce, LockedAxes, ReadMassProperties,
    Sensor, Velocity,
};
use bevy_transform::{
    components::{GlobalTransform, Transform},
    TransformBundle,
};

use crate::{
    collision::TrackCollisions,
    level::InLevel,
    status_effect::StatusBundle,
    time::{Frame, FrameCounter},
    AbilityOffset, FootOffset, Health, Kind, Object, Shootable, FORWARD, PLAYER_R,
};

use super::properties::NeutrinoBallProps;

#[derive(Component)]
pub struct NeutrinoBall {
    pub accel_numerator: f32,
    pub radius: f32,
    pub effect_radius: f32,
    pub activates_at: Frame,
}

pub fn neutrino_ball(
    commands: &mut Commands,
    props: &NeutrinoBallProps,
    transform: &Transform,
    velocity: &Velocity,
    ability_offset: &AbilityOffset,
    counter: &FrameCounter,
) {
    let mut ball_transform = *transform;
    let dir = transform.rotation * FORWARD;

    ball_transform.translation =
        transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ability_offset.to_vec();

    let ball_velocity = velocity.linvel + dir * props.speed;

    commands.spawn((
        Object {
            transform: ball_transform,
            global_transform: GlobalTransform::default(),
            collider: Collider::ball(props.radius),
            foot_offset: (-props.radius).into(),
            mass_props: ColliderMassProperties::Density(100_000.0),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: Velocity::linear(ball_velocity),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            mass: ReadMassProperties::default(),
            kind: Kind::NeutrinoBall,
            in_level: InLevel,
            statuses: StatusBundle::default(),
        },
        NeutrinoBall {
            accel_numerator: props.accel_numerator(),
            radius: props.radius,
            effect_radius: props.effect_radius,
            activates_at: counter.at(props.activation_delay),
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
    tick_counter: Res<FrameCounter>,
    mut neutrino_q: Query<
        (Entity, &NeutrinoBall, &FootOffset),
        Without<NeutrinoBallGravityFieldSpawned>,
    >,
) {
    for (entity, ball, foot_offset) in &mut neutrino_q {
        if ball.activates_at.before_now(&tick_counter) {
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
    mut target_q: Query<(&mut ExternalForce, &Transform, &ReadMassProperties)>,
) {
    for (field, global_transform, colliding) in &neutrino_q {
        for &target in &colliding.targets {
            let Ok((mut force, target_transform, mass)) = target_q.get_mut(target) else {
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

            force.force += f * dir;
        }
    }
}
