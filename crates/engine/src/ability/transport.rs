use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query},
};
use bevy_math::Vec2;
use bevy_rapier3d::prelude::{Collider, ExternalForce, LockedAxes, RigidBody, Sensor, Velocity};
use bevy_transform::{components::Transform, TransformBundle};

use super::properties::TransportProps;
use crate::{
    collision::{TrackCollisionBundle, TrackCollisions},
    level::{Floor, InLevel},
    movement::{DesiredMove, MaxSpeed},
    status_effect::{StatusProps, TimeDilation},
    time::Dur,
    Health, Kind, MassBundle, Object, Target, To2d, To3d,
};

#[derive(Component)]
pub struct TransportBeam {
    pub target: Entity,
    pub delay: Dur,
    pub activates_in: Dur,
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
) {
    let mut transform = Transform::from_translation(transform.translation);
    transform.translation.y = 0.0;
    commands.spawn((
        Object {
            transform: TransformBundle::from_transform(transform),
            collider: Collider::cylinder(props.height * 0.5, props.radius),
            body: RigidBody::Dynamic,
            kind: Kind::TransportBeam,
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Y,
            velocity: Velocity::default(),
            in_level: InLevel,
            foot_offset: 0.0.into(),
            mass: MassBundle::new(10_000.0),
            force: ExternalForce::default(),
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::on(),
        },
        TransportBeam {
            target: entity,
            delay: props.delay,
            activates_in: props.delay,
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
    mut query: Query<(Entity, &mut TransportBeam), Without<ActiveTransportBeam>>,
) {
    for (entity, mut beam) in &mut query {
        // A transport beam originates from the ship above, so it doesn't dilate.
        if beam.activates_in.tick(&TimeDilation::NONE) {
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
            // TODO: We'll likely want to account for altitude difference, or just not allow
            // targeting inside a wall.
            target_transform.translation += delta.to_3d(0.0);
        }
    }
}
