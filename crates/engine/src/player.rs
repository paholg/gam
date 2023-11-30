use bevy_ecs::{component::Component, system::Commands};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{Collider, Friction, LockedAxes, RigidBody};

use crate::{
    ability::Abilities, lifecycle::ENERGY_REGEN, Ally, Character, Cooldowns, Energy, Health, Kind,
    Object, Player, Shootable, Target, PLAYER_R,
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
                        collider: Collider::capsule(
                            Vec3::new(0.0, PLAYER_R, 0.0),
                            Vec3::new(0.0, 1.0 + PLAYER_R, 0.0),
                            PLAYER_R,
                        ),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Player,
                        ..Default::default()
                    },
                    max_speed: Default::default(),
                    impulse: Default::default(),
                    force: Default::default(),
                    friction: Friction::default(),
                    status_effects: Default::default(),
                    shootable: Shootable,
                    abilities: self.abilities.clone(),
                    cooldowns: Cooldowns::new(&self.abilities),
                    desired_movement: Default::default(),
                },
            ))
            .id();
        tracing::debug!(?id, "Spawning player");
    }
}
