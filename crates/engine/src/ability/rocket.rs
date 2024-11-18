use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy_app::Plugin;
use bevy_app::Startup;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::query::With;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::Commands;
use bevy_ecs::system::In;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use super::cooldown::Cooldown;
use super::explosion::ExplosionCallback;
use super::explosion::ExplosionKind;
use super::explosion::ExplosionProps;
use super::noop_ability;
use super::Ability;
use super::AbilityId;
use super::AbilityMap;
use super::Left;
use super::Right;
use super::Side;
use super::SideEnum;
use crate::collision::TrackCollisionBundle;
use crate::collision::TrackCollisions;
use crate::level::InLevel;
use crate::lifecycle::DeathCallback;
use crate::movement::DesiredMove;
use crate::movement::MaxSpeed;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::time::TIMESTEP;
use crate::AbilityOffset;
use crate::Energy;
use crate::GameSet;
use crate::Health;
use crate::MassBundle;
use crate::Object;
use crate::Shootable;
use crate::Target;
use crate::To2d;
use crate::FORWARD;
use crate::PLAYER_R;
use crate::SCHEDULE;

pub struct RocketPlugin;
impl Plugin for RocketPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(RocketProps::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (
                    (cooldown_system::<Left>, cooldown_system::<Right>).in_set(GameSet::Reset),
                    (tracking_system, collision_system).in_set(GameSet::Stuff),
                ),
            );
    }
}

#[derive(Resource)]
pub struct RocketProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub gcd: Dur,
    /// Max turn per tick, in radians.
    pub turning_radius: f32,
    pub capsule_radius: f32,
    pub capsule_length: f32,
    pub health: f32,
    pub max_speed: MaxSpeed,
    /// How much energy the rocket has.
    pub energy: f32,
    /// How much energy the rocket spends every frame to move.
    pub energy_cost: f32,
    pub explosion: ExplosionProps,
    pub mass: f32,
}

impl Default for RocketProps {
    fn default() -> Self {
        Self {
            cost: 30.0,
            cooldown: Dur::new(30),
            gcd: Dur::new(30),
            turning_radius: PI * 0.03,
            capsule_radius: 0.05,
            capsule_length: 0.14,
            health: 3.0,
            max_speed: MaxSpeed {
                accel: 1800.0,
                speed: 8.0,
            },
            energy: 10.0,
            energy_cost: 0.2,
            explosion: ExplosionProps {
                min_radius: 0.2,
                max_radius: 1.2,
                duration: Dur::new(15),
                damage: 0.6,
                force: 300.0,
                kind: ExplosionKind::SeekerRocket,
            },
            mass: 2.0,
        }
    }
}

fn register(world: &mut World) {
    let id = AbilityId::from("rocket");

    let left = (
        Ability::new(world, fire::<Left>, setup::<Left>),
        Ability::new(world, noop_ability, noop_ability),
    );
    let right = (
        Ability::new(world, fire::<Right>, setup::<Right>),
        Ability::new(world, noop_ability, noop_ability),
    );

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();

    ability_map.register_arm(SideEnum::Left, id.clone(), left.0, left.1);
    ability_map.register_arm(SideEnum::Right, id, right.0, right.1);
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
    entity: Entity,
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
    props: Res<RocketProps>,
    explosion_callback: Res<ExplosionCallback>,
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

    let mut transform = *user.transform;
    let dir = transform.rotation * FORWARD;
    // TODO: If the rocket spawns inside a wall, no one will be hurt by its
    // explosion.
    transform.translation = transform.translation
        + dir * (PLAYER_R + props.capsule_length * 2.0)
        + user.ability_offset.to_vec();

    commands.spawn((
        Object {
            transform: transform
                .with_scale(Vec3::new(
                    props.capsule_radius,
                    props.capsule_radius,
                    props.capsule_length * 0.5,
                ))
                .into(),
            collider: Collider::capsule_z(1.0, 1.0),
            foot_offset: (-props.capsule_radius).into(),
            mass: MassBundle::new(props.mass),
            body: RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: *user.velocity,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::on(),
        },
        Health::new(props.health),
        Energy::new(props.energy, 0.0),
        Rocket {
            shooter: user.entity,
            radius: props.capsule_radius,
            turning_radius: props.turning_radius,
            energy_cost: props.energy_cost,
        },
        props.explosion,
        Shootable,
        props.max_speed,
        DeathCallback::new(explosion_callback.system),
        DesiredMove {
            can_fly: true,
            ..Default::default()
        },
        Sensor,
    ));
}

#[derive(Component)]
pub struct Rocket {
    pub shooter: Entity,
    pub radius: f32,
    pub turning_radius: f32,
    pub energy_cost: f32,
}

fn tracking_system(
    mut query: Query<(
        &Rocket,
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

fn collision_system(
    mut rocket_q: Query<(&mut Health, &TrackCollisions, &Velocity, &mut Transform), With<Rocket>>,
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
