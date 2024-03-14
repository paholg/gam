use std::marker::PhantomData;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::WorldQuery,
    system::{Command, Commands, In, Query, Res, ResMut},
    world::World,
};
use bevy_rapier3d::prelude::Velocity;
use bevy_reflect::TypeUuid;
use bevy_transform::components::Transform;
use bevy_utils::Uuid;

use crate::{
    multiplayer::Action, status_effect::TimeDilation, time::Dur, AbilityOffset, Energy, Health,
    FORWARD, PLAYER_R,
};

use super::{
    bullet::{Bullet, BulletSpawner},
    cooldown::Cooldown,
    properties::{AbilityProps, GunProps},
    Ability, AbilityCommand, AbilityMap, Left, Right, Side,
};

const NAME: &'static str = "gun";

pub fn setup(mut map: ResMut<AbilityMap>) {
    map.register_arm(GunBuilder::<Left>::default(), NAME.into(), Action::LeftArm);
    map.register_arm(
        GunBuilder::<Right>::default(),
        NAME.into(),
        Action::RightArm,
    );
}

pub fn cooldown_system<S: Side>(mut query: Query<(&mut GunResources<S>, &TimeDilation)>) {
    for (mut resources, time_dilation) in &mut query {
        resources.cooldown.tick(time_dilation);
    }
}

#[derive(Component)]
pub struct GunResources<S: Side> {
    cooldown: Cooldown,
    ammo: u32,
    _marker: PhantomData<S>,
}

impl<S: Side> GunResources<S> {
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

#[derive(Debug, Default, TypeUuid)]
#[uuid = "6494ab04-0ef8-44fd-a2cd-55e4e61efffe"]
pub struct GunBuilder<S> {
    _marker: PhantomData<S>,
}

// impl<S: Side> ArmAbilityBuilder for GunBuilder<S> {
//     fn build(
//         &self,
//         user: Entity,
//         commands: &mut Commands,
//         props: &AbilityProps,
//     ) -> (Box<dyn AbilityCommand>, Box<dyn AbilityCommand>) {
//         let gun = Gun::<S> {
//             user,
//             _marker: PhantomData,
//         };
//         let reload = GunReload::<S> {
//             user,
//             _marker: PhantomData,
//         };
//         let resources = GunResources::<S>::new(&props.gun);
//         commands.entity(user).insert(resources);

//         (Box::new(gun), Box::new(reload))
//     }
// }

static GUN: Ability = Ability {
    id: Uuid::try_parse("6494ab04-0ef8-44fd-a2cd-55e4e61efffe").unwrap(),
    name: "gun",
    action: Action::LeftArm,
    system: fire::<Left>,
    secondary: None,
};

#[derive(Debug, Clone, Copy)]
pub struct Gun<S> {
    user: Entity,
    _marker: PhantomData<S>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
struct GunUser<S: Side> {
    gcd: &'static Cooldown,
    transform: &'static Transform,
    velocity: &'static Velocity,
    ability_offset: &'static AbilityOffset,
    gun_resources: &'static mut GunResources<S>,
    time_dilation: &'static TimeDilation,
}
fn fire<S: Side>(
    entity: In<Entity>,
    mut commands: Commands,
    user_q: Query<GunUser<S>>,
    props: Res<AbilityProps>,
) {
    let Ok(mut user) = user_q.get_mut(entity) else {
        return;
    };
    if !user.gcd.is_available(&user.time_dilation) {
        return;
    }

    if !user
        .gun_resources
        .try_use(user.time_dilation, props.cooldown)
    {
        return;
    }

    let props = props.gun;

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

// impl<S: Side> Command for Gun<S> {
//     fn apply(self, world: &mut World) {
//         let props = &world.resource::<AbilityProps>().gun;
//         let cooldown = props.cooldown;
//         let radius = props.radius;
//         let mass = props.mass;
//         let duration = props.duration;
//         let damage = props.damage;
//         let bullet_health = props.bullet_health;
//         let speed = props.speed;
//         let mut user_q = world.query::<User<S>>();
//         let Ok(mut user) = user_q.get_mut(world, self.user) else {
//             return;
//         };

//         if !user.gcd.is_available(&user.time_dilation) {
//             return;
//         }

//         if !user.gun_resources.try_use(user.time_dilation, cooldown) {
//             return;
//         }

//         let dir = user.transform.rotation * FORWARD;
//         let position = user.transform.translation
//             + dir * (PLAYER_R + radius * 2.0)
//             + user.ability_offset.to_vec();
//         let velocity = dir * speed + user.velocity.linvel;
//         BulletSpawner {
//             position,
//             velocity,
//             radius,
//             mass,
//             bullet: Bullet {
//                 shooter: self.user,
//                 expires_in: duration,
//                 damage,
//             },
//             health: Health::new(bullet_health),
//         }
//         .spawn(world);
//     }
// }

// impl<S: Side> AbilityCommand for Gun<S> {
//     fn add_command(&self, commands: &mut Commands) {
//         commands.add(*self);
//     }
// }

#[derive(Debug, Clone, Copy)]
pub struct GunReload<S> {
    user: Entity,
    _marker: PhantomData<S>,
}

impl<S: Side> Command for GunReload<S> {
    fn apply(self, world: &mut World) {
        #[derive(WorldQuery)]
        #[world_query(mutable)]
        struct User<S: Side> {
            gun_resources: &'static mut GunResources<S>,
            energy: &'static mut Energy,
            gcd: &'static mut Cooldown,
            time_dilation: &'static TimeDilation,
        }

        let props = &world.resource::<AbilityProps>().gun;
        let reload_cost = props.reload_cost;
        let reload_gcd = props.reload_gcd;
        let ammo = props.ammo;
        let mut user_q = world.query::<User<S>>();
        let Ok(mut user) = user_q.get_mut(world, self.user) else {
            tracing::error!("No user");
            return;
        };
        if !user.gcd.is_available(&user.time_dilation) {
            return;
        }

        if !user.energy.try_use(reload_cost) {
            return;
        };

        user.gcd.set(reload_gcd);
        user.gun_resources.ammo = ammo;
    }
}

// impl<S: Side> AbilityCommand for GunReload<S> {
//     fn add_command(&self, commands: &mut Commands) {
//         commands.add(*self);
//     }
// }
