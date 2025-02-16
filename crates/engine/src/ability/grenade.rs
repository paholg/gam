use std::f32::consts::PI;
use std::marker::PhantomData;

use bevy_app::Plugin;
use bevy_app::Startup;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::system::Commands;
use bevy_ecs::system::In;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::Friction;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::Restitution;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use super::cooldown::Cooldown;
use super::explosion::ExplosionCallback;
use super::explosion::ExplosionKind;
use super::explosion::ExplosionProps;
use super::Ability;
use super::AbilityId;
use super::AbilityMap;
use super::Left;
use super::NonArmSlot;
use super::Right;
use super::Side;
use super::SideEnum;
use crate::collision::TrackCollisionBundle;
use crate::level::InLevel;
use crate::lifecycle::DeathCallback;
use crate::lifecycle::Lifetime;
use crate::physics::G;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::AbilityOffset;
use crate::Energy;
use crate::Health;
use crate::Libm;
use crate::MassBundle;
use crate::Object;
use crate::Shootable;
use crate::Target;
use crate::To2d;
use crate::To3d;
use crate::FORWARD;
use crate::PLAYER_R;
use crate::SCHEDULE;

/// Calculate the initial velocity of a projectile thrown at 45 degrees up, so
/// that it will land at target.
fn calculate_initial_vel(spawn: Vec3, target: Vec3) -> Velocity {
    let dir_in_plane = target.to_2d() - spawn.to_2d();
    let height_delta = target.y - spawn.y;
    let dist_in_plane = dir_in_plane.length();

    // TODO: These can all be constants at some point. Or generated with a proc-
    // macro or build script.
    // Or maybe we'll make "throw angle" customizable.
    let phi = PI / 12.0;
    let cos_phi = Libm::cos(phi);
    let cos_sq_phi = cos_phi * cos_phi;
    let tan_phi = Libm::tan(phi);

    let v0_sq = dist_in_plane * dist_in_plane * G
        / (2.0 * cos_sq_phi * (dist_in_plane * tan_phi - height_delta));
    let v0 = Libm::sqrt(v0_sq);

    let dir = dir_in_plane.to_3d(dist_in_plane * tan_phi).normalize();
    let linvel = v0 * dir;

    Velocity {
        linvel,
        angvel: Vec3::ZERO,
    }
}

#[derive(Debug, Resource)]
pub struct GrenadeProps<G: Grenade> {
    cost: f32,
    cooldown: Dur,
    gcd: Dur,
    delay: Dur,
    radius: f32,
    health: f32,
    explosion: ExplosionProps,
    mass: f32,
    _marker: PhantomData<G>,
}
impl GrenadeProps<FragGrenade> {
    fn new() -> Self {
        Self {
            cost: 30.0,
            cooldown: Dur::new(60),
            gcd: Dur::new(30),
            delay: Dur::new(120),
            radius: 0.07,
            health: 3.0,
            explosion: ExplosionProps {
                min_radius: 0.3,
                max_radius: 1.8,
                duration: Dur::new(15),
                damage: 0.6,
                force: 400.0,
                kind: ExplosionKind::FragGrenade,
            },
            mass: 1.5,
            _marker: PhantomData,
        }
    }
}

impl GrenadeProps<HealGrenade> {
    fn new() -> Self {
        Self {
            cost: 50.0,
            cooldown: Dur::new(60),
            gcd: Dur::new(30),
            delay: Dur::new(120),
            radius: 0.05,
            health: 3.0,
            explosion: ExplosionProps {
                min_radius: 0.2,
                max_radius: 1.2,
                duration: Dur::new(15),
                damage: -1.5,
                force: 0.0,
                kind: ExplosionKind::HealGrenade,
            },
            mass: 1.0,
            _marker: PhantomData,
        }
    }
}

pub struct GrenadePlugin;
impl Plugin for GrenadePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(GrenadeProps::<FragGrenade>::new())
            .insert_resource(GrenadeProps::<HealGrenade>::new())
            .add_systems(Startup, (register::<FragGrenade>, register::<HealGrenade>))
            .add_systems(
                SCHEDULE,
                (
                    cooldown_system::<Left, FragGrenade>,
                    cooldown_system::<Right, FragGrenade>,
                    cooldown_system::<Left, HealGrenade>,
                    cooldown_system::<Right, HealGrenade>,
                ),
            );
    }
}

pub trait Grenade: Send + Sync + Sized + 'static + Component {
    fn id() -> AbilityId;

    fn new(props: &GrenadeProps<Self>) -> Self;
    fn explosion_radius(&self) -> f32;
}

fn register<G: Grenade>(world: &mut World) {
    let id = G::id();
    let left = Ability::new(world, fire::<Left, G>, setup::<Left, G>);
    let right = Ability::new(world, fire::<Right, G>, setup::<Right, G>);
    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();
    ability_map.register(NonArmSlot::Shoulder(SideEnum::Left), id.clone(), left);
    ability_map.register(NonArmSlot::Shoulder(SideEnum::Right), id.clone(), right);
}

fn cooldown_system<S: Side, G: Grenade>(mut query: Query<(&mut Resources<S, G>, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup<S: Side, G: Grenade>(entity: In<Entity>, mut commands: Commands) {
    commands
        .entity(*entity)
        .try_insert(Resources::<S, G>::new());
}

#[derive(Component)]
struct Resources<S: Side, G: Grenade> {
    cooldown: Cooldown,
    _marker: PhantomData<(S, G)>,
}
impl<S: Side, G: Grenade> Resources<S, G> {
    fn new() -> Self {
        Self {
            cooldown: Cooldown::new(),
            _marker: PhantomData,
        }
    }
}

#[derive(Component, Default)]
pub struct FragGrenade {
    explosion_radius: f32,
}
impl Grenade for FragGrenade {
    fn id() -> AbilityId {
        AbilityId::from("frag_grenade")
    }

    fn new(props: &GrenadeProps<Self>) -> Self {
        Self {
            explosion_radius: props.explosion.max_radius,
        }
    }

    fn explosion_radius(&self) -> f32 {
        self.explosion_radius
    }
}

#[derive(Component, Default)]
pub struct HealGrenade {
    explosion_radius: f32,
}
impl Grenade for HealGrenade {
    fn id() -> AbilityId {
        AbilityId::from("heal_grenade")
    }

    fn new(props: &GrenadeProps<Self>) -> Self {
        Self {
            explosion_radius: props.explosion.max_radius,
        }
    }

    fn explosion_radius(&self) -> f32 {
        self.explosion_radius
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct FireQuery<S: Side, G: Grenade> {
    gcd: &'static mut Cooldown,
    energy: &'static mut Energy,
    transform: &'static Transform,
    ability_offset: &'static AbilityOffset,
    resources: &'static mut Resources<S, G>,
    time_dilation: &'static TimeDilation,
    target: &'static Target,
}

fn fire<S: Side, G: Grenade>(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut user_q: Query<FireQuery<S, G>>,
    props: Res<GrenadeProps<G>>,
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

    let dir = user.transform.rotation * FORWARD;
    let position = user.transform.translation
        + dir * (PLAYER_R + props.radius + 0.01)
        + user.ability_offset.to_vec();
    let vel = calculate_initial_vel(position, user.target.0.to_3d(props.radius));

    commands.spawn((
        Object {
            transform: Transform::from_translation(position)
                .with_scale(Vec3::splat(props.radius)),
            collider: Collider::ball(1.0),
            foot_offset: (-props.radius).into(),
            mass: MassBundle::new(props.mass),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: vel,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::off(),
        },
        props.explosion,
        Shootable,
        G::new(&props),
        Friction {
            coefficient: 100.0,
            ..Default::default()
        },
        Restitution {
            coefficient: 0.0,
            ..Default::default()
        },
        Lifetime::new(props.delay),
        DeathCallback::new(explosion_callback.system),
        Health::new(props.health),
    ));
}
