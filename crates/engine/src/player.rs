use bevy_ecs::{component::Component, system::Commands};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ExternalForce, Friction, LockedAxes, RigidBody, Velocity,
};
use bevy_transform::components::Transform;

use crate::{
    ability::Abilities,
    collision::TrackCollisionBundle,
    level::InLevel,
    lifecycle::ENERGY_REGEN,
    status_effect::{Charge, StatusBundle},
    Ally, Character, Cooldowns, Energy, Health, Kind, MassBundle, Object, Player, Shootable,
    Target, ABILITY_Y, PLAYER_HEIGHT, PLAYER_MASS, PLAYER_R,
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
                        transform: Transform::from_xyz(0.0, PLAYER_HEIGHT * 0.5, 0.0).into(),
                        mass: MassBundle::new(PLAYER_MASS),
                        velocity: Velocity::zero(),
                        force: ExternalForce::default(),
                        in_level: InLevel,
                        statuses: StatusBundle::default(),
                        collisions: TrackCollisionBundle::off(),
                    },
                    max_speed: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    abilities: self.abilities.clone(),
                    cooldowns: Cooldowns::new(),
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
