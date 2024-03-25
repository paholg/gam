use std::fmt::Debug;

use bevy_ecs::{
    entity::Entity,
    system::{In, IntoSystem, Resource, SystemId},
    world::{FromWorld, World},
};
use bevy_reflect::TypePath;
use bevy_utils::HashMap;
use serde::{Deserialize, Serialize};

pub mod bullet;
pub mod cooldown;
pub mod gravity_ball;
pub mod grenade;
pub mod gun;
pub mod properties;
pub mod seeker_rocket;
pub mod speed_up;
pub mod transport;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AbilityId(String);

impl From<&str> for AbilityId {
    fn from(value: &str) -> Self {
        AbilityId(value.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ArmAbilitySlot {
    LeftArm,
    RightArm,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AbilitySlot {
    LeftShoulder,
    RightShoulder,
    Head,
    Legs,
}

#[derive(Resource)]
pub struct AbilityMap {
    noop: Ability,
    noop_arm: ArmAbility,
    map: HashMap<AbilitySlot, HashMap<AbilityId, Ability>>,
    arm_map: HashMap<ArmAbilitySlot, HashMap<AbilityId, ArmAbility>>,
}

impl AbilityMap {
    pub fn register(&mut self, slot: AbilitySlot, id: AbilityId, ability: Ability) {
        let action_map = self.map.entry(slot).or_default();
        if action_map.get(&id).is_some() {
            panic!("Duplicate abilities for action {slot:?}, id {id:?}");
        }
        action_map.insert(id, ability);
    }

    pub fn register_arm(&mut self, slot: ArmAbilitySlot, id: AbilityId, ability: ArmAbility) {
        let action_map = self.arm_map.entry(slot).or_default();
        if action_map.get(&id).is_some() {
            panic!("Duplicate abilities for action {slot:?}, id {id:?}");
        }
        action_map.insert(id, ability);
    }

    pub fn get(&self, slot: AbilitySlot, id: &AbilityId) -> &Ability {
        match self.map.get(&slot).and_then(|m| m.get(id)) {
            Some(ability) => ability,
            None => {
                tracing::error!("Missing ability for slot {slot:?}, id {id:?}");
                &self.noop
            }
        }
    }

    pub fn get_arm(&self, slot: ArmAbilitySlot, id: &AbilityId) -> &ArmAbility {
        match self.arm_map.get(&slot).and_then(|m| m.get(id)) {
            Some(ability) => ability,
            None => {
                tracing::error!("Missing ability for {id:?}");
                &self.noop_arm
            }
        }
    }
}

impl FromWorld for AbilityMap {
    fn from_world(world: &mut World) -> Self {
        let noop = Ability::new(world, noop_ability, noop_ability);
        let noop_arm = ArmAbility::new(world, noop_ability, noop_ability, noop_ability);
        AbilityMap {
            noop,
            noop_arm,
            map: Default::default(),
            arm_map: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct ArmAbility {
    /// System to run when this ability is added to an Entity.
    pub setup: SystemId<Entity>,
    /// Main system when this ability is used.
    pub system: SystemId<Entity>,
    /// If this ability has a secondary function, this is it.
    pub secondary: SystemId<Entity>,
}

impl ArmAbility {
    pub fn new<Marker1, Marker2, Marker3>(
        world: &mut World,
        system: impl IntoSystem<Entity, (), Marker1> + 'static,
        setup: impl IntoSystem<Entity, (), Marker2> + 'static,
        secondary: impl IntoSystem<Entity, (), Marker3> + 'static,
    ) -> Self {
        let system = world.register_system(system);
        let setup = world.register_system(setup);
        let secondary = world.register_system(secondary);

        Self {
            system,
            setup,
            secondary,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Ability {
    /// System to run when this ability is added to an Entity.
    pub setup: SystemId<Entity>,
    /// Main system when this ability is used.
    pub system: SystemId<Entity>,
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
            system,
            setup: setup_system,
        }
    }
}

pub trait Side: Debug + Default + Send + Sync + Clone + Copy + 'static {}
#[derive(Debug, Copy, Clone, Default, TypePath)]
pub struct Left;
#[derive(Debug, Copy, Clone, Default, TypePath)]
pub struct Right;

impl Side for Left {}
impl Side for Right {}

fn noop_ability(_: In<Entity>) {}
