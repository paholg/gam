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
use bevy_math::Vec2;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use super::cooldown::Cooldown;
use super::Ability;
use super::AbilityId;
use super::AbilityMap;
use crate::collision::TrackCollisionBundle;
use crate::collision::TrackCollisions;
use crate::level::Floor;
use crate::level::InLevel;
use crate::movement::DesiredMove;
use crate::movement::MaxSpeed;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::Energy;
use crate::GameSet;
use crate::Health;
use crate::MassBundle;
use crate::Object;
use crate::Target;
use crate::To2d;
use crate::To3d;
use crate::SCHEDULE;

pub struct TransportBeamPlugin;
impl Plugin for TransportBeamPlugin {
    fn build(&self, app: &mut bevy_app::App) {
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
            destination: user.target.0,
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

#[derive(Component)]
pub struct TransportBeam {
    pub target: Entity,
    pub delay: Dur,
    pub activates_in: Dur,
    pub radius: f32,
    pub height: f32,
    pub destination: Vec2,
}

fn move_system(
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
            let delta = q.beam.destination - q.transform.translation.to_2d();
            for &target in &q.collisions.targets {
                let Ok(mut target_transform) = target_q.get_mut(target) else {
                    continue;
                };
                // TODO: We'll likely want to account for altitude difference, or just not allow
                // targeting inside a wall.
                target_transform.translation += delta.to_3d(0.0);
            }
        }
    }
}
