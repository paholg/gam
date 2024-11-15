use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Commands;
use bevy_ecs::system::SystemId;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::CoefficientCombineRule;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::Friction;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;
use serde::Deserialize;
use serde::Serialize;

use crate::ability::cooldown::Cooldown;
use crate::ability::AbilityId;
use crate::ability::AbilityMap;
use crate::ability::NonArmSlot;
use crate::ability::SideEnum;
use crate::collision::TrackCollisionBundle;
use crate::level::InLevel;
use crate::lifecycle::ENERGY_REGEN;
use crate::status_effect::StatusProps;
use crate::Ally;
use crate::Character;
use crate::CharacterMarker;
use crate::Energy;
use crate::Health;
use crate::Kind;
use crate::MassBundle;
use crate::Object;
use crate::Player;
use crate::Shootable;
use crate::Target;
use crate::ABILITY_Y;
use crate::CONTACT_SKIN;
use crate::PLAYER_HEIGHT;
use crate::PLAYER_MASS;
use crate::PLAYER_R;

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

        commands.run_system_with_input(left_arm.0.setup, entity);
        commands.run_system_with_input(left_arm.1.setup, entity);
        commands.run_system_with_input(right_arm.0.setup, entity);
        commands.run_system_with_input(right_arm.1.setup, entity);
        commands.run_system_with_input(left_shoulder.setup, entity);
        commands.run_system_with_input(right_shoulder.setup, entity);
        commands.run_system_with_input(legs.setup, entity);
        commands.run_system_with_input(head.setup, entity);

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
    let half = (height * 0.5 - radius) * Vec3::Y;
    debug_assert!(half.y > 0.0);
    Collider::capsule(-half, half, radius)
}
