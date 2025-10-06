use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, In, SystemId},
    },
    math::Vec3,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ExternalForce, Friction, LockedAxes, RigidBody, Velocity,
};
use serde::{Deserialize, Serialize};

use crate::{
    ability::{cooldown::Cooldown, AbilityId, AbilityMap, NonArmSlot, SideEnum},
    collision::TrackCollisionBundle,
    level::InLevel,
    lifecycle::ENERGY_REGEN,
    status_effect::StatusProps,
    Ally, Character, Energy, Health, MassBundle, Object, Player, Shootable, Target, ABILITY_Y,
    CONTACT_SKIN, PLAYER_HEIGHT, PLAYER_MASS, PLAYER_R,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbilityIds {
    pub left_arm: AbilityId,
    pub right_arm: AbilityId,
    pub left_shoulder: AbilityId,
    pub right_shoulder: AbilityId,
    pub legs: AbilityId,
    pub head: AbilityId,
}

impl AbilityIds {
    pub fn build(&self, map: &AbilityMap, commands: &mut Commands, entity: Entity) -> Abilities {
        let left_arm = map.get_arm(SideEnum::Left, &self.left_arm);
        let right_arm = map.get_arm(SideEnum::Right, &self.right_arm);
        let left_shoulder = map.get(NonArmSlot::Shoulder(SideEnum::Left), &self.left_shoulder);
        let right_shoulder = map.get(NonArmSlot::Shoulder(SideEnum::Right), &self.right_shoulder);
        let legs = map.get(NonArmSlot::Legs, &self.legs);
        let head = map.get(NonArmSlot::Head, &self.head);

        commands.run_system_with(left_arm.0.setup, entity);
        commands.run_system_with(left_arm.1.setup, entity);
        commands.run_system_with(right_arm.0.setup, entity);
        commands.run_system_with(right_arm.1.setup, entity);
        commands.run_system_with(left_shoulder.setup, entity);
        commands.run_system_with(right_shoulder.setup, entity);
        commands.run_system_with(legs.setup, entity);
        commands.run_system_with(head.setup, entity);

        Abilities {
            left_arm: left_arm.0.fire,
            left_arm_secondary: left_arm.1.fire,
            right_arm: right_arm.0.fire,
            right_arm_secondary: right_arm.1.fire,
            left_shoulder: left_shoulder.fire,
            right_shoulder: right_shoulder.fire,
            legs: legs.fire,
            head: head.fire,
        }
    }
}

#[derive(Component, Debug)]
pub struct Abilities {
    pub left_arm: SystemId<In<Entity>, ()>,
    pub left_arm_secondary: SystemId<In<Entity>, ()>,
    pub right_arm: SystemId<In<Entity>, ()>,
    pub right_arm_secondary: SystemId<In<Entity>, ()>,
    pub left_shoulder: SystemId<In<Entity>, ()>,
    pub right_shoulder: SystemId<In<Entity>, ()>,
    pub legs: SystemId<In<Entity>, ()>,
    pub head: SystemId<In<Entity>, ()>,
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
                    object: Object {
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        transform: Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.5, 0.0),
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
                    contact_skin: CONTACT_SKIN,
                    health: Health::new(100.0),
                    energy: Energy::new(100.0, ENERGY_REGEN),
                    max_speed: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    global_cooldown: Cooldown::new(),
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                },
            ))
            .id();
        let abilities = self.ability_ids.build(ability_map, commands, id);
        commands.entity(id).insert(abilities);
        tracing::debug!(?id, "Spawning player");
    }
}

pub fn character_collider(radius: f32, height: f32) -> Collider {
    let half = (height * 0.5 - radius) * Vec3::Y;
    debug_assert!(half.y > 0.0);
    Collider::capsule(-half, half, radius)
}
