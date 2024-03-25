use std::{f32::consts::PI, fmt::Debug, marker::PhantomData};

use bevy_app::{Plugin, Startup};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::QueryData,
    schedule::IntoSystemConfigs,
    system::{Commands, In, Query, Res, Resource},
    world::World,
};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{
    Collider, ExternalForce, Friction, LockedAxes, Restitution, Velocity,
};
use bevy_reflect::TypePath;
use bevy_transform::components::Transform;

use super::{
    cooldown::Cooldown, properties::ExplosionProps, Ability, AbilityId, AbilityMap, AbilitySlot,
    Left, Right, Side,
};
use crate::{
    collision::TrackCollisionBundle,
    death_callback::{DeathCallback, ExplosionCallback, ExplosionKind},
    level::InLevel,
    physics::G,
    status_effect::{StatusProps, TimeDilation},
    time::Dur,
    AbilityOffset, Energy, GameSet, Health, Kind, Libm, MassBundle, Object, Shootable, Target,
    To2d, To3d, FORWARD, PLAYER_R, SCHEDULE,
};

pub struct GrenadePlugin;

impl Plugin for GrenadePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(GrenadeProps::<Frag>::default())
            .insert_resource(GrenadeProps::<Heal>::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (
                    (cooldown::<Left>, cooldown::<Right>).in_set(GameSet::Reset),
                    explode.in_set(GameSet::Stuff),
                ),
            );
    }
}

fn register(world: &mut World) {
    let heal_id = AbilityId::from("heal_grenade");
    let frag_id = AbilityId::from("frag_grenade");

    let heal_left = Ability::new(world, fire::<Heal, Left>, setup::<Left>);
    let heal_right = Ability::new(world, fire::<Heal, Right>, setup::<Right>);

    let frag_left = Ability::new(world, fire::<Frag, Left>, setup::<Left>);
    let frag_right = Ability::new(world, fire::<Frag, Right>, setup::<Right>);

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();
    ability_map.register(AbilitySlot::LeftShoulder, heal_id.clone(), heal_left);
    ability_map.register(AbilitySlot::RightShoulder, heal_id, heal_right);

    ability_map.register(AbilitySlot::LeftShoulder, frag_id.clone(), frag_left);
    ability_map.register(AbilitySlot::RightShoulder, frag_id, frag_right);
}

fn cooldown<S: Side>(mut query: Query<(&mut Resources<S>, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup<S: Side>(entity: In<Entity>, mut commands: Commands) {
    commands
        .entity(*entity)
        .try_insert(Resources::<S>::default());
}

pub trait GrenadeType: Debug + Default + Send + Sync + Clone + Copy + 'static {}
#[derive(Debug, Copy, Clone, Default, TypePath)]
pub struct Frag;
#[derive(Debug, Copy, Clone, Default, TypePath)]
pub struct Heal;

impl GrenadeType for Frag {}
impl GrenadeType for Heal {}

#[derive(Debug, Resource)]
pub struct GrenadeProps<G: GrenadeType> {
    cost: f32,
    cooldown: Dur,
    gcd: Dur,
    delay: Dur,
    pub radius: f32,
    kind: GrenadeKind,
    health: f32,
    pub explosion: ExplosionProps,
    mass: f32,
    _marker: PhantomData<G>,
}

impl Default for GrenadeProps<Frag> {
    fn default() -> Self {
        Self {
            cost: 30.0,
            cooldown: Dur::new(45),
            gcd: Dur::new(30),
            delay: Dur::new(120),
            radius: 0.07,
            kind: GrenadeKind::Frag,
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

impl Default for GrenadeProps<Heal> {
    fn default() -> Self {
        Self {
            cost: 50.0,
            cooldown: Dur::new(45),
            gcd: Dur::new(30),
            delay: Dur::new(120),
            radius: 0.05,
            kind: GrenadeKind::Heal,
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

#[derive(Component, Default)]
pub struct Resources<S: Side> {
    cooldown: Cooldown,
    _marker: PhantomData<S>,
}

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

#[derive(Debug, Copy, Clone)]
pub enum GrenadeKind {
    Frag,
    Heal,
}

impl From<GrenadeKind> for Kind {
    fn from(value: GrenadeKind) -> Self {
        match value {
            GrenadeKind::Frag => Kind::FragGrenade,
            GrenadeKind::Heal => Kind::HealGrenade,
        }
    }
}

#[derive(Component)]
pub struct Grenade {
    // TODO: Use this field
    #[allow(dead_code)]
    shooter: Entity,
    expires_in: Dur,
    pub kind: GrenadeKind,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct User<S: Side> {
    gcd: &'static mut Cooldown,
    transform: &'static Transform,
    target: &'static Target,
    ability_offset: &'static AbilityOffset,
    resources: &'static mut Resources<S>,
    energy: &'static mut Energy,
    time_dilation: &'static TimeDilation,
}

fn fire<G: GrenadeType, S: Side>(
    In(entity): In<Entity>,
    mut commands: Commands,
    props: Res<GrenadeProps<G>>,
    mut user_q: Query<User<S>>,
) {
    let Ok(mut user) = user_q.get_mut(entity) else {
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

    let dir = user.transform.rotation * FORWARD;
    let position = user.transform.translation
        + dir * (PLAYER_R + props.radius + 0.01)
        + user.ability_offset.to_vec();
    let vel = calculate_initial_vel(position, user.target.0.to_3d(props.radius));

    commands.spawn((
        Object {
            transform: Transform::from_translation(position).into(),
            collider: Collider::ball(props.radius),
            foot_offset: (-props.radius).into(),
            mass: MassBundle::new(props.mass),
            body: bevy_rapier3d::prelude::RigidBody::Dynamic,
            force: ExternalForce::default(),
            velocity: vel,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            kind: props.kind.into(),
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::off(),
        },
        Shootable,
        Grenade {
            expires_in: props.delay,
            shooter: entity,
            kind: props.kind,
        },
        Friction {
            coefficient: 100.0,
            ..Default::default()
        },
        Restitution {
            coefficient: 0.0,
            ..Default::default()
        },
        DeathCallback::Explosion(ExplosionCallback {
            props: props.explosion,
        }),
        Health::new(props.health),
    ));
}

fn explode(mut query: Query<(&mut Grenade, &mut Health, &TimeDilation)>) {
    for (mut grenade, mut health, dilation) in &mut query {
        if grenade.expires_in.tick(dilation) {
            health.die();
        }
    }
}
