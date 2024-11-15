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

use super::bullet::Bullet;
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
use crate::Health;
use crate::FORWARD;
use crate::PLAYER_R;
use crate::SCHEDULE;

pub struct GunPlugin;
impl Plugin for GunPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(GunProps::default())
            .add_systems(Startup, register)
            .add_systems(
                SCHEDULE,
                (cooldown_system::<Left>, cooldown_system::<Right>).in_set(GameSet::Reset),
            );
    }
}

#[derive(Debug, Resource)]
pub struct GunProps {
    ammo: u32,
    cooldown: Dur,
    duration: Dur,
    pub speed: f32,
    pub radius: f32,
    damage: f32,
    mass: f32,
    bullet_health: f32,
    reload_cost: f32,
    reload_gcd: Dur,
    reload_cd: Dur,
}

impl Default for GunProps {
    fn default() -> Self {
        Self {
            ammo: 30,
            cooldown: Dur::new(5),
            duration: Dur::new(600),
            speed: 12.0,
            radius: 0.03,
            damage: 2.0,
            bullet_health: 1.0,
            mass: 0.25,
            reload_cost: 50.0,
            reload_gcd: Dur::new(30),
            reload_cd: Dur::new(120),
        }
    }
}

fn register(world: &mut World) {
    let id = AbilityId::from("gun");

    let left = (
        Ability::new(world, fire::<Left>, setup::<Left>),
        Ability::new(world, reload::<Left>, noop_ability),
    );
    let right = (
        Ability::new(world, fire::<Right>, setup::<Right>),
        Ability::new(world, reload::<Right>, noop_ability),
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

fn setup<S: Side>(entity: In<Entity>, mut commands: Commands, props: Res<GunProps>) {
    commands
        .entity(*entity)
        .try_insert(Resources::<S>::new(&props));
}

#[derive(Component)]
pub struct Resources<S: Side> {
    cooldown: Cooldown,
    ammo: u32,
    _marker: PhantomData<S>,
}
impl<S: Side> Resources<S> {
    pub fn new(props: &GunProps) -> Self {
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
struct FireQuery<S: Side> {
    gcd: &'static mut Cooldown,
    transform: &'static Transform,
    velocity: &'static Velocity,
    ability_offset: &'static AbilityOffset,
    gun_resources: &'static mut Resources<S>,
    time_dilation: &'static TimeDilation,
}
fn fire<S: Side>(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut user_q: Query<FireQuery<S>>,
    props: Res<GunProps>,
) {
    let Ok(mut user) = user_q.get_mut(entity) else {
        return;
    };
    if !user.gcd.is_available(user.time_dilation) {
        return;
    }

    if !user
        .gun_resources
        .try_use(user.time_dilation, props.cooldown)
    {
        return;
    }
    user.gcd.set(props.cooldown);

    let dir = user.transform.rotation * FORWARD;
    let position = user.transform.translation
        + dir * (PLAYER_R + props.radius * 2.0)
        + user.ability_offset.to_vec();
    let velocity = dir * props.speed + user.velocity.linvel;

    BulletSpawner {
        position,
        velocity,
        radius: props.radius,
        mass: props.mass,
        bullet: Bullet {
            shooter: entity,
            expires_in: props.duration,
            damage: props.damage,
        },
        health: Health::new(props.bullet_health),
    }
    .spawn(&mut commands);
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ReloadQuery<S: Side> {
    gun_resources: &'static mut Resources<S>,
    energy: &'static mut Energy,
    gcd: &'static mut Cooldown,
    time_dilation: &'static TimeDilation,
}
fn reload<S: Side>(entity: In<Entity>, mut user_q: Query<ReloadQuery<S>>, props: Res<GunProps>) {
    let Ok(mut user) = user_q.get_mut(*entity) else {
        return;
    };

    if !user.gcd.is_available(user.time_dilation) {
        return;
    }

    if !user.energy.try_use(props.reload_cost) {
        return;
    };

    user.gcd.set(props.reload_gcd);
    user.gun_resources.ammo = props.ammo;
    user.gun_resources.cooldown.set(props.reload_cd);
}
