use std::marker::PhantomData;

use bevy_app::{Plugin, Startup};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{QueryData, With, Without},
    schedule::IntoSystemConfigs,
    system::{Commands, In, Query, Res, Resource},
    world::World,
};
use bevy_rapier3d::prelude::{
    Collider, ExternalForce, LockedAxes, ReadMassProperties, Sensor, Velocity,
};
use bevy_transform::components::Transform;

use super::{cooldown::Cooldown, Ability, AbilityId, AbilityMap, AbilitySlot, Left, Right, Side};
use crate::{
    collision::{TrackCollisionBundle, TrackCollisions},
    level::InLevel,
    status_effect::{StatusProps, TimeDilation},
    time::Dur,
    Energy, FootOffset, GameSet, Health, Kind, Libm, MassBundle, Object, FORWARD, PLAYER_R,
    SCHEDULE,
};

pub struct GravityBallPlugin;

impl Plugin for GravityBallPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(GravityBallProps::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (
                    collision_system.in_set(GameSet::Collision),
                    activation_system.in_set(GameSet::Stuff),
                    cooldown_system::<Left>,
                    cooldown_system::<Right>,
                ),
            );
    }
}

fn register(world: &mut World) {
    let id = AbilityId::from("gravity_ball");

    let left = Ability::new(world, fire::<Left>, setup::<Left>);
    let right = Ability::new(world, fire::<Right>, setup::<Right>);

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();

    ability_map.register(AbilitySlot::LeftShoulder, id.clone(), left);
    ability_map.register(AbilitySlot::RightShoulder, id, right);
}

fn cooldown_system<S: Side>(mut query: Query<(&mut Resources<S>, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup<S: Side>(entity: In<Entity>, mut commands: Commands) {
    commands
        .entity(*entity)
        .try_insert(Resources::<S>::default());
}

#[derive(Debug, Copy, Clone, Resource)]
pub struct GravityBallProps {
    cost: f32,
    cooldown: Dur,
    gcd: Dur,
    pub radius: f32,
    pub effect_radius: f32,
    duration: Dur,
    activation_delay: Dur,
    speed: f32,
    surface_a: f32,
}

impl Default for GravityBallProps {
    fn default() -> Self {
        Self {
            cost: 50.0,
            cooldown: Dur::new(30),
            gcd: Dur::new(30),
            radius: 1.0,
            effect_radius: 3.0,
            duration: Dur::new(240),
            activation_delay: Dur::new(30),
            speed: 3.0,
            surface_a: 35.0,
        }
    }
}

impl GravityBallProps {
    /// The acceleration due to this ball will be the result value, divided by
    /// distance squared.
    pub fn accel_numerator(&self) -> f32 {
        self.surface_a * self.radius * self.radius
    }

    pub fn mass(&self) -> f32 {
        const G: f32 = 6.674e-11;
        // Rather than setting the mass, it's more convenient to set the surface
        // gravity and compute the mass.
        self.accel_numerator() / G
    }
}

#[derive(Component, Default)]
struct Resources<S: Side> {
    cooldown: Cooldown,
    _marker: PhantomData<S>,
}

#[derive(Component)]
pub struct GravityBall {
    accel_numerator: f32,
    surface_a: f32,
    radius: f32,
    effect_radius: f32,
    activates_in: Dur,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct User<S: Side> {
    gcd: &'static mut Cooldown,
    transform: &'static Transform,
    velocity: &'static Velocity,
    foot_offset: &'static FootOffset,
    resources: &'static mut Resources<S>,
    energy: &'static mut Energy,
    time_dilation: &'static TimeDilation,
}

fn fire<S: Side>(
    entity: In<Entity>,
    mut commands: Commands,
    mut user_q: Query<User<S>>,
    props: Res<GravityBallProps>,
) {
    let Ok(mut user) = user_q.get_mut(*entity) else {
        return;
    };
    if !user.gcd.is_available(user.time_dilation) {
        return;
    }
    if !user.resources.cooldown.is_available(user.time_dilation) {
        return;
    }
    if !user.energy.try_use(props.cost) {
        return;
    }
    user.gcd.set(props.gcd);
    user.resources.cooldown.set(props.cooldown);
    // if !user.resources.try_use(user
    let mut ball_transform = *user.transform;
    let dir = user.transform.rotation * FORWARD;

    // Let's spawn this one at the caster's feet.
    ball_transform.translation =
        user.transform.translation + dir * (PLAYER_R + props.radius) + user.foot_offset.to_vec();

    let ball_velocity = user.velocity.linvel + dir * props.speed;

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
        GravityBall {
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
pub struct GravityBallActivated;

fn activation_system(
    mut commands: Commands,
    mut gravity_q: Query<(Entity, &mut GravityBall, &TimeDilation), Without<GravityBallActivated>>,
) {
    for (entity, mut ball, time_dilation) in &mut gravity_q {
        if ball.activates_in.tick(time_dilation) {
            commands.entity(entity).insert(GravityBallActivated);
        }
    }
}

fn collision_system(
    gravity_q: Query<(&GravityBall, &Transform, &TrackCollisions), With<GravityBallActivated>>,
    mut target_q: Query<(
        &mut ExternalForce,
        &Transform,
        &ReadMassProperties,
        &TimeDilation,
    )>,
) {
    for (ball, transform, colliding) in &gravity_q {
        for &target in &colliding.targets {
            let Ok((mut force, target_transform, mass, dilation)) = target_q.get_mut(target) else {
                // TODO: Exclude `Floor` before this. Or maybe it doesn't
                // matter.
                // tracing::warn!(?target, "Gravity ball hit target, but not in query");
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
