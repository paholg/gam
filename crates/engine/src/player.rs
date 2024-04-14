use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, SystemId},
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ExternalForce, Friction, LockedAxes, RigidBody, Velocity,
};
use bevy_transform::components::Transform;
use serde::{Deserialize, Serialize};

use crate::{
    ability::{cooldown::Cooldown, AbilityId, AbilityMap, AbilitySlot, ArmAbilitySlot},
    collision::TrackCollisionBundle,
    level::InLevel,
    lifecycle::ENERGY_REGEN,
    status_effect::StatusProps,
    Ally, Character, CharacterMarker, Energy, Health, Kind, MassBundle, Object, Player, Shootable,
    Target, ABILITY_Y, PLAYER_HEIGHT, PLAYER_MASS, PLAYER_R,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityIds {
    pub left_arm: AbilityId,
    pub right_arm: AbilityId,
    pub left_shoulder: AbilityId,
    pub right_shoulder: AbilityId,
    pub legs: AbilityId,
    pub head: AbilityId,
}

impl AbilityIds {
    fn build(&self, map: &AbilityMap, commands: &mut Commands, entity: Entity) -> Abilities {
        let left_arm = map.get_arm(ArmAbilitySlot::LeftArm, &self.left_arm);
        let right_arm = map.get_arm(ArmAbilitySlot::RightArm, &self.right_arm);
        let left_shoulder = map.get(AbilitySlot::LeftShoulder, &self.left_shoulder);
        let right_shoulder = map.get(AbilitySlot::RightShoulder, &self.right_shoulder);
        let legs = map.get(AbilitySlot::Legs, &self.legs);
        let head = map.get(AbilitySlot::Head, &self.head);

        commands.run_system_with_input(left_arm.setup, entity);
        commands.run_system_with_input(right_arm.setup, entity);
        commands.run_system_with_input(left_shoulder.setup, entity);
        commands.run_system_with_input(right_shoulder.setup, entity);
        commands.run_system_with_input(legs.setup, entity);
        commands.run_system_with_input(head.setup, entity);

        Abilities {
            left_arm: left_arm.system,
            left_arm_secondary: left_arm.secondary,
            right_arm: right_arm.system,
            right_arm_secondary: right_arm.secondary,
            left_shoulder: left_shoulder.system,
            right_shoulder: right_shoulder.system,
            legs: legs.system,
            head: head.system,
        }
    }
}

#[derive(Component, Debug)]
pub struct Abilities {
    pub left_arm: SystemId<Entity, ()>,
    pub left_arm_secondary: SystemId<Entity, ()>,
    pub right_arm: SystemId<Entity, ()>,
    pub right_arm_secondary: SystemId<Entity, ()>,
    pub left_shoulder: SystemId<Entity, ()>,
    pub right_shoulder: SystemId<Entity, ()>,
    pub legs: SystemId<Entity, ()>,
    pub head: SystemId<Entity, ()>,
}

#[derive(Debug, Component)]
pub struct PlayerInfo {
    pub handle: Player,
    pub ability_ids: AbilityIds,
}

impl PlayerInfo {
    pub fn spawn_player(&self, commands: &mut Commands, ability_map: &AbilityMap) {
        let id = commands
            .spawn((
                Target::default(),
                self.handle,
                Ally,
                Character {
                    health: Health::new(100.0),
                    energy: Energy::new(100.0, ENERGY_REGEN),
                    object: Object {
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Player,
                        transform: Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.5, 0.0).into(),
                        mass: MassBundle::new(PLAYER_MASS),
                        velocity: Velocity::zero(),
                        force: ExternalForce::default(),
                        in_level: InLevel,
                        statuses: StatusProps {
                            thermal_mass: 1.0,
                            capacitance: 1.0,
                        }
                        .into(),
                        collisions: TrackCollisionBundle::off(),
                    },
                    max_speed: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    global_cooldown: Cooldown::new(),
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                    marker: CharacterMarker,
                },
            ))
            .id();
        let abilities = self.ability_ids.build(ability_map, commands, id);
        commands.entity(id).insert(abilities);

        tracing::debug!(?id, "Spawning player");
    }
}

pub fn character_collider(radius: f32, height: f32) -> Collider {
    Collider::cylinder(height * 0.5, radius)
}
