use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res},
};
use bevy_math::Vec2;
use bevy_rapier3d::prelude::{ActiveEvents, Collider, LockedAxes, RigidBody, Sensor, Velocity};
use bevy_transform::{components::Transform, TransformBundle};

use crate::{
    collision::TrackCollisions,
    level::{Floor, InLevel},
    movement::{DesiredMove, MaxSpeed},
    time::{Dur, Frame, FrameCounter},
    Health, Kind, Target, To2d, To3d,
};

use super::properties::TransportProps;

#[derive(Component)]
pub struct TransportBeam {
    pub target: Entity,
    pub delay: Dur,
    pub activation_time: Frame,
    pub radius: f32,
    pub height: f32,
    pub destination: Vec2,
}

pub fn transport(
    commands: &mut Commands,
    entity: Entity,
    props: &TransportProps,
    transform: &Transform,
    target: &Target,
    tick_counter: &FrameCounter,
) {
    let mut transform = Transform::from_translation(transform.translation);
    transform.translation.y = 0.0;
    commands.spawn((
        TransformBundle::from_transform(transform),
        Collider::cylinder(props.height * 0.5, props.radius),
        RigidBody::Dynamic,
        Kind::TransportBeam,
        LockedAxes::TRANSLATION_LOCKED_Y,
        Velocity::default(),
        InLevel,
        TransportBeam {
            target: entity,
            delay: props.delay,
            activation_time: tick_counter.at(props.delay),
            radius: props.radius,
            height: props.height,
            destination: target.0,
        },
        MaxSpeed {
            accel: props.accel,
            speed: props.speed,
        },
        DesiredMove {
            dir: Vec2::ZERO,
            can_fly: true,
        },
        Sensor,
        TrackCollisions::default(),
        ActiveEvents::COLLISION_EVENTS,
    ));
}

pub fn move_system(
    mut query: Query<(&mut DesiredMove, &Transform, &mut TransportBeam)>,
    target_q: Query<&Transform>,
) {
    for (mut desired_move, transform, beam) in &mut query {
        let Ok(target_transform) = target_q.get(beam.target) else {
            desired_move.reset();
            continue;
        };

        desired_move.dir = (target_transform.translation.to_2d() - transform.translation.to_2d())
            .clamp_length_max(1.0);
    }
}

#[derive(Component)]
pub struct ActiveTransportBeam;

pub fn activation_system(
    mut commands: Commands,
    query: Query<(Entity, &TransportBeam), Without<ActiveTransportBeam>>,
    tick_counter: Res<FrameCounter>,
) {
    for (entity, beam) in &query {
        if beam.activation_time.before_now(&tick_counter) {
            commands
                .entity(entity)
                .insert((Health::new(0.0), ActiveTransportBeam));
        }
    }
}

pub fn collision_system(
    beam_q: Query<(&TrackCollisions, &TransportBeam, &Transform), With<ActiveTransportBeam>>,
    mut target_q: Query<&mut Transform, (Without<TransportBeam>, Without<Floor>)>,
) {
    for (collisions, beam, transform) in &beam_q {
        let delta = beam.destination - transform.translation.to_2d();
        for &target in &collisions.targets {
            let Ok(mut target_transform) = target_q.get_mut(target) else {
                continue;
            };
            // TODO: We'll likely want to account for altitude difference, or just not allow targeting inside a wall.
            target_transform.translation += delta.to_3d(0.0);
        }
    }
}
