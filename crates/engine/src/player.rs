use bevy_ecs::{component::Component, entity::Entity, system::Commands};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ExternalForce, Friction, LockedAxes, RigidBody, Velocity,
};
use bevy_transform::components::Transform;
use bevy_utils::Uuid;
use serde::{Deserialize, Serialize};

use crate::{
    ability::{cooldown::Cooldown, properties::AbilityProps, AbilityCommand, AbilityMap},
    collision::TrackCollisionBundle,
    level::InLevel,
    lifecycle::ENERGY_REGEN,
    status_effect::StatusProps,
    Ally, Character, CharacterMarker, Energy, Health, Kind, MassBundle, Object, Player, Shootable,
    Target, ABILITY_Y, PLAYER_HEIGHT, PLAYER_MASS, PLAYER_R,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AbilityIds {
    pub left_arm: Uuid,
    pub left_arm_secondary: Uuid,
    pub right_arm: Uuid,
    pub right_arm_secondary: Uuid,
    pub left_shoulder: Uuid,
    pub right_shoulder: Uuid,
    pub legs: Uuid,
    pub head: Uuid,
}

impl AbilityIds {
    pub fn build(
        &self,
        map: &AbilityMap,
        user: Entity,
        commands: &mut Commands,
        props: &AbilityProps,
    ) -> Option<Abilities> {
        let (left_arm, left_arm_secondary) = map.get_arm(self.left_arm, user, commands, props)?;
        let (right_arm, right_arm_secondary) =
            map.get_arm(self.right_arm, user, commands, props)?;
        let left_shoulder = map.get(self.left_shoulder, user, commands, props)?;
        let right_shoulder = map.get(self.right_shoulder, user, commands, props)?;
        let legs = map.get(self.legs, user, commands, props)?;
        let head = map.get(self.head, user, commands, props)?;
        Some(Abilities {
            left_arm,
            left_arm_secondary,
            right_arm,
            right_arm_secondary,
            left_shoulder,
            right_shoulder,
            legs,
            head,
        })
    }
}

#[derive(Component)]
pub struct Abilities {
    pub left_arm: AbilityCommand,
    pub left_arm_secondary: AbilityCommand,
    pub right_arm: AbilityCommand,
    pub right_arm_secondary: AbilityCommand,
    pub left_shoulder: AbilityCommand,
    pub right_shoulder: AbilityCommand,
    pub legs: AbilityCommand,
    pub head: AbilityCommand,
}

#[derive(Debug, Component)]
pub struct PlayerInfo {
    pub handle: Player,
    pub ability_ids: AbilityIds,
}

impl PlayerInfo {
    pub fn spawn_player(
        &self,
        commands: &mut Commands,
        props: &AbilityProps,
        ability_map: &AbilityMap,
    ) {
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
        let Some(abilities) = self.ability_ids.build(ability_map, id, commands, props) else {
            tracing::error!("Missing abilities for player");
            return;
        };
        commands.entity(id).insert(abilities);

        tracing::debug!(?id, "Spawning player");
    }
}

pub fn character_collider(radius: f32, height: f32) -> Collider {
    Collider::cylinder(height * 0.5, radius)
}
