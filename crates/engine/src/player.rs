use bevy_ecs::{component::Component, system::Commands};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{Collider, LockedAxes, RigidBody};

use crate::{
    ability::Abilities, lifecycle::ENERGY_REGEN, Ally, Character, Cooldowns, Energy, Health,
    Object, Player, Shootable, Target, DAMPING, PLAYER_R,
};

#[derive(Debug, Component)]
pub struct PlayerInfo {
    pub handle: Player,
    pub abilities: Abilities,
}

impl PlayerInfo {
    pub fn spawn_player(&self, commands: &mut Commands) {
        commands.spawn((
            Target::default(),
            self.handle,
            Ally,
            Character {
                health: Health::new(100.0),
                energy: Energy::new(100.0, ENERGY_REGEN),
                damping: DAMPING,
                object: Object {
                    collider: Collider::capsule(
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(0.0, 0.0, 2.0),
                        PLAYER_R,
                    ),
                    body: RigidBody::Dynamic,
                    locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                    ..Default::default()
                },
                max_speed: Default::default(),
                impulse: Default::default(),
                status_effects: Default::default(),
                shootable: Shootable,
                abilities: self.abilities.clone(),
                cooldowns: Cooldowns::new(&self.abilities),
            },
        ));
    }
}
