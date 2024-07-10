use std::{f32::consts::PI, marker::PhantomData};

use bevy_app::{Plugin, Startup};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{QueryData, With},
    schedule::IntoSystemConfigs,
    system::{Commands, In, Query, Res, Resource},
    world::World,
};
use bevy_rapier3d::prelude::{Collider, ExternalForce, LockedAxes, Sensor, Velocity};
use bevy_transform::components::Transform;

use super::{
    cooldown::Cooldown, properties::ExplosionProps, Ability, AbilityId, AbilityMap, Left, Right,
    Side,
};
use crate::{
    collision::{TrackCollisionBundle, TrackCollisions},
    death_callback::{DeathCallback, ExplosionCallback, ExplosionKind},
    level::InLevel,
    movement::{DesiredMove, MaxSpeed},
    status_effect::{StatusProps, TimeDilation},
    time::{Dur, TIMESTEP},
    AbilityOffset, Energy, GameSet, Health, Kind, MassBundle, Object, Shootable, Target, To2d,
    FORWARD, PLAYER_R, SCHEDULE,
};

pub struct SeekerRocketPlugin;

impl Plugin for SeekerRocketPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(SeekerRocketProps::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (
                    collision_system.in_set(GameSet::Collision),
                    tracking_system.in_set(GameSet::Stuff),
                    cooldown_system::<Left>,
                    cooldown_system::<Right>,
                ),
            );
    }
}

fn register(world: &mut World) {
    let id = AbilityId::from("seeker_rocket");

    let left = Ability::new(world, fire::<Left>, setup::<Left>);
    let right = Ability::new(world, fire::<Right>, setup::<Right>);

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();

    ability_map.register(super::AbilitySlot::LeftShoulder, id.clone(), left);
    ability_map.register(super::AbilitySlot::RightShoulder, id, right);
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

#[derive(Debug, Resource)]
pub struct SeekerRocketProps {
    pub cost: f32,
    pub cooldown: Dur,
    gcd: Dur,
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

impl Default for SeekerRocketProps {
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

#[derive(Component, Default)]
struct Resources<S: Side> {
    cooldown: Cooldown,
    _marker: PhantomData<S>,
}

#[derive(Component)]
struct SeekerRocket {
    shooter: Entity,
    radius: f32,
    turning_radius: f32,
    energy_cost: f32,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct User<S: Side> {
    gcd: &'static mut Cooldown,
    transform: &'static Transform,
    velocity: &'static Velocity,
    ability_offset: &'static AbilityOffset,
    resources: &'static mut Resources<S>,
    energy: &'static mut Energy,
    time_dilation: &'static TimeDilation,
}

fn fire<S: Side>(
    entity: In<Entity>,
    mut commands: Commands,
    mut user_q: Query<User<S>>,
    props: Res<SeekerRocketProps>,
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

    let mut rocket_transform = *user.transform;
    let dir = rocket_transform.rotation * FORWARD;
    // TODO: If the rocket spawns inside a wall, no one will be hurt by its
    // explosion.
    rocket_transform.translation = user.transform.translation
        + dir * (PLAYER_R + props.capsule_length * 2.0)
        + user.ability_offset.to_vec();

    commands.spawn((
        Object {
            transform: rocket_transform.into(),
            collider: Collider::capsule_z(props.capsule_length * 0.5, props.capsule_radius),
            foot_offset: (-props.capsule_radius).into(),
            mass: MassBundle::new(props.mass),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: *user.velocity,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
            kind: Kind::SeekerRocket,
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
        SeekerRocket {
            shooter: *entity,
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
        Sensor,
    ));
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TrackingQuery {
    rocket: &'static SeekerRocket,
    transform: &'static mut Transform,
    desired_move: &'static mut DesiredMove,
    energy: &'static mut Energy,
    locked_axes: &'static mut LockedAxes,
}

fn tracking_system(mut query: Query<TrackingQuery>, target_query: Query<&Target>) {
    for mut rocket in query.iter_mut() {
        if rocket.energy.try_use(rocket.rocket.energy_cost) {
            let Ok(target) = target_query.get(rocket.rocket.shooter) else {
                continue;
            };
            let target = target.0;

            let facing = rocket.transform.forward().to_2d();

            let desired_rotation =
                facing.angle_between(target - rocket.transform.translation.to_2d());
            let rotation =
                desired_rotation.clamp(-rocket.rocket.turning_radius, rocket.rocket.turning_radius);

            rocket.transform.rotate_y(rotation);

            // Rockets always go forward.
            rocket.desired_move.dir = (rocket.transform.rotation * FORWARD).to_2d();
        } else {
            // Unlock y translation, so it can fall.
            *rocket.locked_axes = LockedAxes::ROTATION_LOCKED;
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CollisionQuery {
    health: &'static mut Health,
    colliding: &'static TrackCollisions,
    transform: &'static mut Transform,
    velocity: &'static Velocity,
}

fn collision_system(
    mut query: Query<CollisionQuery, With<SeekerRocket>>,
    shootable_q: Query<(), With<Shootable>>,
) {
    for mut rocket in &mut query {
        let should_live = rocket.colliding.targets.is_empty()
            || rocket
                .colliding
                .targets
                .iter()
                .all(|&t| shootable_q.get(t).is_err());

        if !should_live {
            rocket.health.die();
            // If a rocket hits a wall, it will be inside it when it explodes,
            // damaging no one. So, we just move back a frame.
            rocket.transform.translation -= rocket.velocity.linvel * TIMESTEP;
        }
    }
}
