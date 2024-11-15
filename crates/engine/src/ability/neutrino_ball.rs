use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::query::Without;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_hierarchy::BuildChildren;
use bevy_rapier3d::prelude::ActiveEvents;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::ReadMassProperties;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::bundles::TransformBundle;
use bevy_transform::components::GlobalTransform;
use bevy_transform::components::Transform;

use super::properties::NeutrinoBallProps;
use crate::collision::TrackCollisionBundle;
use crate::collision::TrackCollisions;
use crate::level::InLevel;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::AbilityOffset;
use crate::FootOffset;
use crate::Health;
use crate::Kind;
use crate::MassBundle;
use crate::Object;
use crate::Shootable;
use crate::FORWARD;
use crate::PLAYER_R;

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
    ability_offset: &AbilityOffset,
) {
    let mut ball_transform = *transform;
    let dir = transform.rotation * FORWARD;

    // Let's spawn this one at the caster's feet.
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
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::off(),
        },
        NeutrinoBall {
            accel_numerator: props.accel_numerator(),
            surface_a: props.surface_a,
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

#[derive(QueryData)]
#[query_data(mutable)]
pub struct ActivationQuery {
    entity: Entity,
    neutrino_ball: &'static mut NeutrinoBall,
    foot_offset: &'static FootOffset,
    time_dilation: &'static TimeDilation,
}

pub fn activation_system(
    mut commands: Commands,
    mut neutrino_q: Query<ActivationQuery, Without<NeutrinoBallGravityFieldSpawned>>,
) {
    for mut q in &mut neutrino_q {
        if q.neutrino_ball.activates_in.tick(q.time_dilation) {
            commands
                .entity(q.entity)
                .insert(NeutrinoBallGravityFieldSpawned)
                .with_children(|builder| {
                    builder.spawn((
                        NeutrinoBallGravityField {
                            accel_numerator: q.neutrino_ball.accel_numerator,
                        },
                        *q.foot_offset,
                        TransformBundle::default(),
                        ActiveEvents::COLLISION_EVENTS,
                        TrackCollisions::default(),
                        Collider::ball(q.neutrino_ball.effect_radius),
                        Sensor,
                    ));
                });
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct CollisionQuery {
    field: &'static NeutrinoBallGravityField,
    global_transform: &'static GlobalTransform,
    colliding: &'static TrackCollisions,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct TargetQuery {
    force: &'static mut ExternalForce,
    transform: &'static Transform,
    mass: &'static ReadMassProperties,
    dilation: &'static TimeDilation,
}

pub fn collision_system(neutrino_q: Query<CollisionQuery>, mut target_q: Query<TargetQuery>) {
    for q in &neutrino_q {
        for &colliding_entity in &q.colliding.targets {
            let Ok(mut target) = target_q.get_mut(colliding_entity) else {
                // TODO: Exclude `Floor` before this. Or maybe it doesn't
                // matter.
                // tracing::warn!(?target, "Neutrino ball hit target, but not in query");
                continue;
            };
            let translation = q.global_transform.translation();
            let d2 = translation.distance_squared(target.transform.translation);

            let a = q.field.accel_numerator / d2;
            let f = target.mass.mass * a;

            let mut dir = (translation - target.transform.translation).normalize();
            // Let's keep it from letting you fly up, for now.
            dir.y = 0.0;

            target.force.force += f * target.dilation.factor() * dir;
        }
    }
}
