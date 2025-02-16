use std::marker::PhantomData;

use bevy_app::Plugin;
use bevy_app::Startup;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::query::Without;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::Commands;
use bevy_ecs::system::In;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use bevy_hierarchy::BuildChildren;
use bevy_hierarchy::ChildBuild;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::ActiveEvents;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::ReadMassProperties;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::GlobalTransform;
use bevy_transform::components::Transform;

use super::cooldown::Cooldown;
use super::Ability;
use super::AbilityId;
use super::AbilityMap;
use super::Left;
use super::NonArmSlot;
use super::Right;
use super::Side;
use super::SideEnum;
use crate::collision::TrackCollisionBundle;
use crate::collision::TrackCollisions;
use crate::level::InLevel;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::AbilityOffset;
use crate::Energy;
use crate::FootOffset;
use crate::GameSet;
use crate::Health;
use crate::MassBundle;
use crate::Object;
use crate::Shootable;
use crate::FORWARD;
use crate::PLAYER_R;
use crate::SCHEDULE;

pub struct GravityBallPlugin;
impl Plugin for GravityBallPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(GravityBallProps::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (
                    (cooldown_system::<Left>, cooldown_system::<Right>).in_set(GameSet::Reset),
                    (activation_system, collision_system).in_set(GameSet::Stuff),
                ),
            );
    }
}

#[derive(Debug, Resource)]
struct GravityBallProps {
    cost: f32,
    cooldown: Dur,
    gcd: Dur,
    radius: f32,
    effect_radius: f32,
    duration: Dur,
    activation_delay: Dur,
    speed: f32,
    surface_a: f32,
}

impl Default for GravityBallProps {
    fn default() -> Self {
        Self {
            cost: 50.0,
            cooldown: Dur::new(60),
            gcd: Dur::new(30),
            radius: 0.3,
            effect_radius: 3.0,
            duration: Dur::new(240),
            activation_delay: Dur::new(30),
            speed: 3.0,
            surface_a: 300.0,
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

fn register(world: &mut World) {
    let id = AbilityId::from("gravity_ball");

    let left = Ability::new(world, fire::<Left>, setup::<Left>);
    let right = Ability::new(world, fire::<Right>, setup::<Right>);

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();

    ability_map.register(NonArmSlot::Shoulder(SideEnum::Left), id.clone(), left);
    ability_map.register(NonArmSlot::Shoulder(SideEnum::Right), id.clone(), right);
}

fn cooldown_system<S: Side>(mut query: Query<(&mut Resources<S>, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup<S: Side>(entity: In<Entity>, mut commands: Commands) {
    commands.entity(*entity).try_insert(Resources::<S>::new());
}

#[derive(Component)]
struct Resources<S: Side> {
    cooldown: Cooldown,
    _marker: PhantomData<S>,
}
impl<S: Side> Resources<S> {
    fn new() -> Self {
        Self {
            cooldown: Cooldown::new(),
            _marker: PhantomData,
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct FireQuery<S: Side> {
    gcd: &'static mut Cooldown,
    energy: &'static mut Energy,
    transform: &'static Transform,
    velocity: &'static Velocity,
    ability_offset: &'static AbilityOffset,
    resources: &'static mut Resources<S>,
    time_dilation: &'static TimeDilation,
}
fn fire<S: Side>(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut user_q: Query<FireQuery<S>>,
    props: Res<GravityBallProps>,
) {
    let Ok(mut user) = user_q.get_mut(entity) else {
        return;
    };

    if !user.gcd.is_available(user.time_dilation) {
        return;
    }

    if user.resources.cooldown.is_available(user.time_dilation) && user.energy.try_use(props.cost) {
        user.resources.cooldown.set(props.cooldown);
        user.gcd.set(props.gcd);
    } else {
        return;
    }

    let dir = user.transform.rotation * FORWARD;
    let position =
        user.transform.translation + dir * (PLAYER_R + props.radius) + user.ability_offset.to_vec();
    let velocity = dir * props.speed + user.velocity.linvel;

    commands.spawn((
        Object {
            transform: Transform::from_translation(position)
                .with_scale(Vec3::splat(props.radius)),
            collider: Collider::ball(1.0),
            foot_offset: (-props.radius).into(),
            mass: MassBundle::new(props.mass()),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: Velocity::linear(velocity),
            // TODO: This sinks through the floor sometimes, despite translation being locked.
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::off(),
        },
        GravityBall {
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
pub struct GravityBall {
    pub accel_numerator: f32,
    pub surface_a: f32,
    pub radius: f32,
    pub effect_radius: f32,
    pub activates_in: Dur,
}

#[derive(Component)]
struct GravityBallGravityFieldSpawned;

#[derive(Component)]
pub struct GravityBallGravityField {
    accel_numerator: f32,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ActivationQuery {
    entity: Entity,
    gravity_ball: &'static mut GravityBall,
    foot_offset: &'static FootOffset,
    time_dilation: &'static TimeDilation,
    transform: &'static Transform,
}

fn activation_system(
    mut commands: Commands,
    mut gravity_q: Query<ActivationQuery, Without<GravityBallGravityFieldSpawned>>,
) {
    for mut q in &mut gravity_q {
        if q.gravity_ball.activates_in.tick(q.time_dilation) {
            let parent_scale = q.transform.scale;
            let scale = parent_scale.recip() * q.gravity_ball.effect_radius;
            commands
                .entity(q.entity)
                .insert(GravityBallGravityFieldSpawned)
                .with_children(|builder| {
                    builder.spawn((
                        GravityBallGravityField {
                            accel_numerator: q.gravity_ball.accel_numerator,
                        },
                        *q.foot_offset,
                        Transform::from_scale(scale),
                        ActiveEvents::COLLISION_EVENTS,
                        TrackCollisions::default(),
                        Collider::ball(1.0),
                        Sensor,
                    ));
                });
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CollisionQuery {
    field: &'static GravityBallGravityField,
    global_transform: &'static GlobalTransform,
    colliding: &'static TrackCollisions,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TargetQuery {
    force: &'static mut ExternalForce,
    transform: &'static Transform,
    mass: &'static ReadMassProperties,
    dilation: &'static TimeDilation,
}

fn collision_system(gravity_q: Query<CollisionQuery>, mut target_q: Query<TargetQuery>) {
    for q in &gravity_q {
        for &colliding_entity in &q.colliding.targets {
            let Ok(mut target) = target_q.get_mut(colliding_entity) else {
                // TODO: Exclude `Floor` before this. Or maybe it doesn't
                // matter.
                // tracing::warn!(?target, "Gravity ball hit target, but not in query");
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
