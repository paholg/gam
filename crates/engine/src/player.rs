use bevy_ecs::{component::Component, system::Commands};
use bevy_rapier3d::prelude::{CoefficientCombineRule, Collider, Friction, LockedAxes, RigidBody};
use bevy_transform::components::Transform;

use crate::{
    ability::Abilities, lifecycle::ENERGY_REGEN, Ally, Character, Cooldowns, Energy, Health, Kind,
    Object, Player, Shootable, Target, ABILITY_Y, PLAYER_HEIGHT, PLAYER_R,
};

#[derive(Debug, Component)]
pub struct PlayerInfo {
    pub handle: Player,
    pub abilities: Abilities,
}

impl PlayerInfo {
    pub fn spawn_player(&self, commands: &mut Commands) {
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
                        transform: Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.5, 0.0),
                        ..Default::default()
                    },
                    max_speed: Default::default(),
                    impulse: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    status_effects: Default::default(),
                    shootable: Shootable,
                    abilities: self.abilities.clone(),
                    cooldowns: Cooldowns::new(&self.abilities),
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                },
            ))
            .id();
        tracing::debug!(?id, "Spawning player");
    }
}

pub fn character_collider(radius: f32, height: f32) -> Collider {
    Collider::cylinder(height * 0.5, radius)
}
