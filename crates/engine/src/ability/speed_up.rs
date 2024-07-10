use bevy_app::{Plugin, Startup};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::QueryData,
    system::{Commands, In, Query, Res, Resource},
    world::World,
};

use super::{cooldown::Cooldown, Ability, AbilityId, AbilityMap};
use crate::{status_effect::TimeDilation, time::Dur, Energy, SCHEDULE};

pub struct SpeedUpPlugin;

impl Plugin for SpeedUpPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(SpeedUpProps::default())
            .add_systems(Startup, register)
            .add_systems(SCHEDULE, cooldown);
    }
}

fn register(world: &mut World) {
    let id = AbilityId::from("speed_up");
    let ability = Ability::new(world, fire, setup);

    let mut ability_map = world.get_resource_mut::<AbilityMap>().unwrap();
    ability_map.register(super::AbilitySlot::Legs, id, ability);
}

fn cooldown(mut query: Query<(&mut Resources, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

fn setup(entity: In<Entity>, mut commands: Commands) {
    commands.entity(*entity).try_insert(Resources::default());
}

#[derive(Debug, Resource)]
struct SpeedUpProps {
    cost: f32,
    cooldown: Dur,
    gcd: Dur,
    duration: Dur,
    amount: f32,
}

impl Default for SpeedUpProps {
    fn default() -> Self {
        Self {
            cost: 2.0,
            cooldown: Dur::new(1),
            gcd: Dur::new(30),
            duration: Dur::new(1),
            amount: 1.0,
        }
    }
}

#[derive(Component, Default)]
struct Resources {
    cooldown: Cooldown,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct User {
    gcd: &'static mut Cooldown,
    resources: &'static mut Resources,
    time_dilation: &'static mut TimeDilation,
    energy: &'static mut Energy,
}

fn fire(entity: In<Entity>, mut query: Query<User>, props: Res<SpeedUpProps>) {
    let Ok(mut user) = query.get_mut(*entity) else {
        return;
    };

    if !user.gcd.is_available(&user.time_dilation) {
        return;
    }
    if !user.resources.cooldown.is_available(&user.time_dilation) {
        return;
    }
    if !user.energy.try_use(props.cost) {
        return;
    }

    user.gcd.set(props.gcd);
    user.resources.cooldown.set(props.cooldown);

    user.time_dilation.add_effect(props.amount, props.duration);
}
