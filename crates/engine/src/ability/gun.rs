use std::marker::PhantomData;

use bevy_app::Plugin;
use bevy_app::Startup;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::QueryData;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::Commands;
use bevy_ecs::system::In;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::Resource;
use bevy_ecs::world::World;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use super::bullet::BulletProps;
use super::bullet::BulletSpawner;
use super::cooldown::Cooldown;
use super::noop_ability;
use super::Ability;
use super::AbilityId;
use super::AbilityMap;
use super::Left;
use super::Right;
use super::Side;
use super::SideEnum;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::AbilityOffset;
use crate::Energy;
use crate::GameSet;
use crate::FORWARD;
use crate::PLAYER_R;
use crate::SCHEDULE;

pub struct GunPlugin;
impl Plugin for GunPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(GunProps::<StandardGun>::default())
            .insert_resource(GunProps::<FireGun>::default())
            .insert_resource(GunProps::<ColdGun>::default())
            .add_systems(
                Startup,
                (
                    register::<StandardGun>,
                    register::<FireGun>,
                    register::<ColdGun>,
                ),
            )
            .add_systems(
                SCHEDULE,
                (
                    cooldown_system::<Left, StandardGun>,
                    cooldown_system::<Right, StandardGun>,
                    cooldown_system::<Left, FireGun>,
                    cooldown_system::<Right, FireGun>,
                    cooldown_system::<Left, ColdGun>,
                    cooldown_system::<Right, ColdGun>,
                )
                    .in_set(GameSet::Reset),
            );
    }
}

pub trait GunKind: Send + Sync + Sized + 'static + Component {
    fn id() -> AbilityId;
    fn new() -> Self;
}

#[derive(Component, Default)]
pub struct StandardGun;

impl GunKind for StandardGun {
    fn id() -> AbilityId {
        AbilityId::from("gun")
    }

    fn new() -> Self {
        Self
    }
}

#[derive(Component, Default)]
pub struct FireGun;

impl GunKind for FireGun {
    fn id() -> AbilityId {
        AbilityId::from("fire_gun")
    }

    fn new() -> Self {
        Self
    }
}

#[derive(Component, Default)]
pub struct ColdGun;

impl GunKind for ColdGun {
    fn id() -> AbilityId {
        AbilityId::from("cold_gun")
    }

    fn new() -> Self {
        Self
    }
}

#[derive(Debug, Resource)]
pub struct GunProps<G: GunKind> {
    ammo: u32,
    cooldown: Dur,
    pub speed: f32,
    reload_cost: f32,
    reload_gcd: Dur,
    reload_cd: Dur,
    pub bullet: BulletProps,
    _marker: PhantomData<G>,
}

impl Default for GunProps<StandardGun> {
    fn default() -> Self {
        Self {
            ammo: 100,
            cooldown: Dur::new(5),
            speed: 12.0,
            reload_cost: 50.0,
            reload_gcd: Dur::new(30),
            reload_cd: Dur::new(120),
            bullet: BulletProps {
                radius: 0.03,
                mass: 0.5,
                health: 1.0,
                lifetime: Dur::new(600),
                damage: 2.0,
                heat: 0.0,
            },
            _marker: PhantomData,
        }
    }
}

impl Default for GunProps<FireGun> {
    fn default() -> Self {
        Self {
            ammo: 100,
            cooldown: Dur::new(5),
            speed: 12.0,
            reload_cost: 50.0,
            reload_gcd: Dur::new(30),
            reload_cd: Dur::new(120),
            bullet: BulletProps {
                radius: 0.05,
                mass: 0.5,
                health: 1.0,
                lifetime: Dur::new(20),
                damage: 0.0,
                heat: 2.0,
            },
            _marker: PhantomData,
        }
    }
}

impl Default for GunProps<ColdGun> {
    fn default() -> Self {
        Self {
            ammo: 100,
            cooldown: Dur::new(5),
            speed: 12.0,
            reload_cost: 50.0,
            reload_gcd: Dur::new(30),
            reload_cd: Dur::new(120),
            bullet: BulletProps {
                radius: 0.03,
                mass: 0.25,
                health: 1.0,
                lifetime: Dur::new(600),
                damage: 0.0,
                heat: -3.0,
            },
            _marker: PhantomData,
        }
    }
}

fn register<G: GunKind>(world: &mut World) {
    let id = G::id();

    let left = (
        Ability::new(world, fire::<Left, G>, setup::<Left, G>),
        Ability::new(world, reload::<Left, G>, noop_ability),
    );
    let right = (
        Ability::new(world, fire::<Right, G>, setup::<Right, G>),
        Ability::new(world, reload::<Right, G>, noop_ability),
    );

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();

    ability_map.register_arm(SideEnum::Left, id.clone(), left.0, left.1);
    ability_map.register_arm(SideEnum::Right, id, right.0, right.1);
}

fn cooldown_system<S: Side, G: GunKind>(mut query: Query<(&mut Resources<S, G>, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup<S: Side, G: GunKind>(entity: In<Entity>, mut commands: Commands, props: Res<GunProps<G>>) {
    commands
        .entity(*entity)
        .try_insert(Resources::<S, G>::new(&props));
}

#[derive(Component)]
pub struct Resources<S: Side, G: GunKind> {
    cooldown: Cooldown,
    ammo: u32,
    _marker: PhantomData<(S, G)>,
}
impl<S: Side, G: GunKind> Resources<S, G> {
    pub fn new(props: &GunProps<G>) -> Self {
        Self {
            cooldown: Cooldown::new(),
            ammo: props.ammo,
            _marker: PhantomData,
        }
    }

    fn try_use(&mut self, time_dilation: &TimeDilation, cooldown: Dur) -> bool {
        if self.ammo > 0 && self.cooldown.is_available(time_dilation) {
            self.ammo -= 1;
            self.cooldown.set(cooldown);
            true
        } else {
            false
        }
    }
}
#[derive(QueryData)]
#[query_data(mutable)]
struct FireQuery<S: Side, G: GunKind> {
    gcd: &'static mut Cooldown,
    transform: &'static Transform,
    velocity: &'static Velocity,
    ability_offset: &'static AbilityOffset,
    resources: &'static mut Resources<S, G>,
    time_dilation: &'static TimeDilation,
}
fn fire<S: Side, G: GunKind>(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut user_q: Query<FireQuery<S, G>>,
    props: Res<GunProps<G>>,
) {
    let Ok(mut user) = user_q.get_mut(entity) else {
        return;
    };
    if !user.gcd.is_available(user.time_dilation) {
        return;
    }

    if !user.resources.try_use(user.time_dilation, props.cooldown) {
        return;
    }
    // user.gcd.set(props.cooldown);

    let dir = user.transform.rotation * FORWARD;
    let position = user.transform.translation
        + dir * (PLAYER_R + props.bullet.radius * 2.0)
        + user.ability_offset.to_vec();
    let velocity = dir * props.speed + user.velocity.linvel;

    BulletSpawner {
        shooter: entity,
        position,
        velocity,
        props: props.bullet,
        gun_kind: G::new(),
    }
    .spawn(&mut commands);
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ReloadQuery<S: Side, G: GunKind> {
    resources: &'static mut Resources<S, G>,
    energy: &'static mut Energy,
    gcd: &'static mut Cooldown,
    time_dilation: &'static TimeDilation,
}
fn reload<S: Side, G: GunKind>(
    entity: In<Entity>,
    mut user_q: Query<ReloadQuery<S, G>>,
    props: Res<GunProps<G>>,
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

    if !user.energy.try_use(props.reload_cost) {
        return;
    };

    user.gcd.set(props.reload_gcd);
    user.resources.ammo = props.ammo;
    user.resources.cooldown.set(props.reload_cd);
}
