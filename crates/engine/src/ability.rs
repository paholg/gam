use bevy_app::Plugin;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::In;
use bevy_ecs::system::IntoSystem;
use bevy_ecs::system::Resource;
use bevy_ecs::system::SystemId;
use bevy_ecs::world::FromWorld;
use bevy_ecs::world::World;
use bevy_reflect::TypePath;
use bevy_utils::HashMap;
use explosion::ExplosionPlugin;
use gravity_ball::GravityBallPlugin;
use grenade::GrenadePlugin;
use gun::GunPlugin;
use rocket::RocketPlugin;
use serde::Deserialize;
use serde::Serialize;
use subenum::subenum;
use transport::TransportBeamPlugin;

pub mod bullet;
pub mod cooldown;
pub mod explosion;
pub mod gravity_ball;
pub mod grenade;
pub mod gun;
pub mod rocket;
pub mod transport;

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins((
            ExplosionPlugin,
            GravityBallPlugin,
            GrenadePlugin,
            GunPlugin,
            RocketPlugin,
            TransportBeamPlugin,
        ));
    }
}

pub trait Side: Default + Send + Sync + Clone + Copy + 'static {}
#[derive(Debug, Copy, Clone, Default, TypePath)]
pub struct Left;
#[derive(Debug, Copy, Clone, Default, TypePath)]
pub struct Right;

impl Side for Left {}
impl Side for Right {}

fn noop_ability(_: In<Entity>) {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideEnum {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[subenum(NonArmSlot)]
pub enum Slot {
    Arm(SideEnum),
    ArmSecondary(SideEnum),
    #[subenum(NonArmSlot)]
    Shoulder(SideEnum),
    #[subenum(NonArmSlot)]
    Legs,
    #[subenum(NonArmSlot)]
    Head,
}

#[derive(Copy, Clone)]
pub struct Ability {
    /// System to run when this ability is added to an Entity.
    pub setup: SystemId<Entity>,
    /// Main system when this ability is used.
    pub fire: SystemId<Entity>,
}

impl Ability {
    pub fn new<Marker1, Marker2>(
        world: &mut World,
        system: impl IntoSystem<Entity, (), Marker1> + 'static,
        setup_system: impl IntoSystem<Entity, (), Marker2> + 'static,
    ) -> Self {
        let system = world.register_system(system);
        let setup_system = world.register_system(setup_system);

        Self {
            fire: system,
            setup: setup_system,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AbilityId(String);

impl Default for AbilityId {
    fn default() -> Self {
        Self("noop".to_string())
    }
}

impl From<&str> for AbilityId {
    fn from(value: &str) -> Self {
        AbilityId(value.into())
    }
}

#[derive(Resource)]
pub struct AbilityMap {
    noop: Ability,
    map: HashMap<Slot, HashMap<AbilityId, Ability>>,
}

impl AbilityMap {
    pub fn register_arm(
        &mut self,
        side: SideEnum,
        id: AbilityId,
        ability: Ability,
        secondary: Ability,
    ) {
        let primary_map = self.map.entry(Slot::Arm(side)).or_default();
        assert!(
            primary_map.get(&id).is_none(),
            "Duplicate abilities for primary arm action {side:?}, id {id:?}"
        );
        primary_map.insert(id.clone(), ability);

        let secondary_map = self.map.entry(Slot::ArmSecondary(side)).or_default();
        assert!(
            secondary_map.get(&id).is_none(),
            "Duplicate abilities for secondary arm action {side:?}, id {id:?}"
        );
        secondary_map.insert(id, secondary);
    }

    pub fn register(&mut self, slot: NonArmSlot, id: AbilityId, ability: Ability) {
        let slot = slot.into();
        let map = self.map.entry(slot).or_default();
        assert!(
            map.get(&id).is_none(),
            "Duplicate abilities for action {slot:?}, id {id:?}"
        );
        map.insert(id, ability);
    }

    pub fn get_arm(&self, side: SideEnum, id: &AbilityId) -> (&Ability, &Ability) {
        let primary = match self.map.get(&Slot::Arm(side)).and_then(|m| m.get(id)) {
            Some(ability) => ability,
            None => {
                if id.0 != "noop" {
                    tracing::error!(
                        "Missing primary ability for primary arm action {side:?}, id {id:?}"
                    );
                }
                &self.noop
            }
        };
        let secondary = match self
            .map
            .get(&Slot::ArmSecondary(side))
            .and_then(|m| m.get(id))
        {
            Some(ability) => ability,
            None => {
                if id.0 != "noop" {
                    tracing::error!(
                        "Missing primary ability for secondary arm action {side:?}, id {id:?}"
                    );
                }
                &self.noop
            }
        };

        (primary, secondary)
    }

    pub fn get(&self, slot: NonArmSlot, id: &AbilityId) -> &Ability {
        let slot: Slot = slot.into();
        match self.map.get(&slot).and_then(|m| m.get(id)) {
            Some(ability) => ability,
            None => {
                if id.0 != "noop" {
                    tracing::error!("Missing ability for slot {slot:?}, id {id:?}");
                }
                &self.noop
            }
        }
    }
}

impl FromWorld for AbilityMap {
    fn from_world(world: &mut World) -> Self {
        let noop = Ability::new(world, noop_ability, noop_ability);
        AbilityMap {
            noop,
            map: Default::default(),
        }
    }
}

// impl Abilities {
//     pub fn new(abilities: Vec<Ability>) -> Self {
//         Self { inner: abilities }
//     }

//     pub fn iter(&self) -> impl Iterator<Item = Ability> + '_ {
//         self.inner.iter().copied()
//     }

//     pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Ability> {
//         self.inner.iter_mut()
//     }
// }

// impl Index<usize> for Abilities {
//     type Output = Ability;

//     fn index(&self, index: usize) -> &Self::Output {
//         &self.inner[index]
//     }
// }

// impl Ability {
//     #[allow(clippy::too_many_arguments)]
//     pub fn fire(
//         &self,
//         commands: &mut Commands,
//         props: &AbilityProps,
//         entity: Entity,
//         energy: &mut Energy,
//         cooldowns: &mut Cooldowns,
//         transform: &Transform,
//         velocity: &Velocity,
//         target: &Target,
//         ability_offset: &AbilityOffset,
//         _foot_offset: &FootOffset,
//         time_dilation: &mut TimeDilation,
//     ) -> bool {
//         // if cooldowns.is_available(self, time_dilation) &&
//         // energy.try_use(props.cost(self)) {     cooldowns.set(*self,
//         // props.cooldown(self)); } else {
//         //     return false;
//         // }

//         // match self {
//         //     Ability::None => (),
//         // }
//         true
//     }
// }

// fn gun(
//     commands: &mut Commands,
//     props: &GunProps,
//     transform: &Transform,
//     velocity: &Velocity,
//     shooter: Entity,
//     ability_offset: &AbilityOffset,
// ) {
//     let dir = transform.rotation * FORWARD;
//     let position =
//         transform.translation + dir * (PLAYER_R + props.radius * 2.0) +
//     ability_offset.to_vec(); let velocity = dir * props.speed +
//     velocity.linvel; BulletSpawner {
//         position,
//         velocity,
//         radius: props.radius,
//         mass: props.mass,
//         bullet: Bullet {
//             shooter,
//             expires_in: props.duration,
//             damage: props.damage,
//         },
//         health: Health::new(props.bullet_health),
//     }
//     .spawn(commands);
// }

// fn shotgun(
//     commands: &mut Commands,
//     props: &ShotgunProps,
//     transform: &Transform,
//     velocity: &Velocity,
//     shooter: Entity,
//     ability_offset: &AbilityOffset,
// ) {
//     for i in 0..props.n_pellets {
//         let idx = i as f32;
//         let n_pellets = props.n_pellets as f32;
//         let relative_angle = (n_pellets * 0.5 - idx) / n_pellets *
//     props.spread;     let relative_angle =
//     Quat::from_rotation_z(relative_angle);     let dir =
//     (transform.rotation * relative_angle) * FORWARD;     let position =
//             transform.translation + dir * (PLAYER_R + props.radius * 2.0) +
//     ability_offset.to_vec();     let velocity = dir * props.speed +
//     velocity.linvel;     BulletSpawner {
//             position,
//             velocity,
//             radius: props.radius,
//             mass: props.mass,
//             bullet: Bullet {
//                 shooter,
//                 expires_in: props.duration,
//                 damage: props.damage,
//             },
//             health: Health::new(props.bullet_health),
//         }
//         .spawn(commands);
//     }
// }
