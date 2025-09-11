use bevy::{
    app::{App, Plugin, Startup},
    ecs::{
        component::Component,
        entity::Entity,
        query::{QueryData, Without},
        resource::Resource,
        schedule::IntoScheduleConfigs,
        system::{Commands, In, Query, Res},
        world::World,
    },
    math::Vec3,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::{Collider, ExternalForce, LockedAxes, RigidBody, Sensor, Velocity};

use super::{cooldown::Cooldown, Ability, AbilityId, AbilityMap};
use crate::{
    collision::{TrackCollisionBundle, TrackCollisions},
    level::{Floor, InLevel},
    movement::{DesiredMove, MaxSpeed},
    status_effect::{StatusProps, TimeDilation},
    time::Dur,
    Energy, GameSet, Health, MassBundle, Object, Target, FORWARD, SCHEDULE, UP,
};

pub struct TransportBeamPlugin;
impl Plugin for TransportBeamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TransportProps::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (
                    cooldown_system.in_set(GameSet::Reset),
                    (move_system, activation_system).in_set(GameSet::Stuff),
                ),
            );
    }
}

#[derive(Resource)]
struct TransportProps {
    cost: f32,
    cooldown: Dur,
    gcd: Dur,
    radius: f32,
    height: f32,
    accel: f32,
    speed: f32,
    delay: Dur,
}

impl Default for TransportProps {
    fn default() -> Self {
        Self {
            cost: 40.0,
            cooldown: Dur::new(90),
            gcd: Dur::new(30),
            radius: 0.5,
            height: 2.0,
            accel: 100.0,
            speed: 3.0,
            delay: Dur::new(90),
        }
    }
}

fn register(world: &mut World) {
    let id = AbilityId::from("transport_beam");
    let ability = Ability::new(world, fire, setup);

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();
    ability_map.register(super::NonArmSlot::Legs, id, ability);
}

fn cooldown_system(mut query: Query<(&mut Resources, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup(entity: In<Entity>, mut commands: Commands) {
    commands.entity(*entity).try_insert(Resources::new());
}

#[derive(Component)]
struct Resources {
    cooldown: Cooldown,
}
impl Resources {
    fn new() -> Self {
        Self {
            cooldown: Cooldown::new(),
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct FireQuery {
    gcd: &'static mut Cooldown,
    energy: &'static mut Energy,
    transform: &'static Transform,
    resources: &'static mut Resources,
    time_dilation: &'static TimeDilation,
    target: &'static Target,
}
fn fire(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut user_q: Query<FireQuery>,
    props: Res<TransportProps>,
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

    let mut transform = Transform::from_translation(user.transform.translation);
    transform.translation.y = 0.0;
    commands.spawn((
        Object {
            transform,
            collider: Collider::cylinder(props.height * 0.5, props.radius),
            body: RigidBody::Dynamic,
            locked_axes: LockedAxes::TRANSLATION_LOCKED_Y,
            velocity: Velocity::default(),
            in_level: InLevel,
            foot_offset: 0.0.into(),
            // TODO: Why does this have mass?
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
            destination: user.target.transform.translation,
        },
        MaxSpeed {
            accel: props.accel,
            speed: props.speed,
        },
        DesiredMove {
            vec: Vec3::ZERO,
            can_fly: true,
        },
        Sensor,
    ));
}

#[derive(Component)]
pub struct TransportBeam {
    pub target: Entity,
    pub delay: Dur,
    pub activates_in: Dur,
    pub radius: f32,
    pub height: f32,
    pub destination: Vec3,
}

fn move_system(
    mut query: Query<(&mut DesiredMove, &mut Transform, &TransportBeam)>,
    target_q: Query<&Transform, Without<TransportBeam>>,
) {
    for (mut desired_move, mut transform, beam) in &mut query {
        let Ok(target_transform) = target_q.get(beam.target) else {
            desired_move.reset();
            continue;
        };

        *transform = transform.looking_at(target_transform.translation, UP);

        let desired_move_dist = (target_transform.translation - transform.translation)
            .length()
            .clamp(0.0, 1.0);

        desired_move.vec = desired_move_dist * FORWARD;
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ActivationQuery {
    entity: Entity,
    beam: &'static mut TransportBeam,
    collisions: &'static TrackCollisions,
    transform: &'static Transform,
}

fn activation_system(
    mut commands: Commands,
    mut query: Query<ActivationQuery>,
    mut target_q: Query<&mut Transform, (Without<TransportBeam>, Without<Floor>)>,
) {
    for mut q in &mut query {
        // A transport beam originates from the ship above, so it doesn't dilate.
        if q.beam.activates_in.tick(&TimeDilation::NONE) {
            commands.entity(q.entity).insert(Health::new(0.0));
            let delta = q.beam.destination - q.transform.translation;
            for &target in &q.collisions.targets {
                let Ok(mut target_transform) = target_q.get_mut(target) else {
                    continue;
                };
                // TODO: We'll likely want to account for altitude difference, or just not allow
                // targeting inside a wall.
                target_transform.translation += delta;
            }
        }
    }
}
