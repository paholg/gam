use bevy_ecs::component::Component;

use crate::ability::Ability;

/// A temporary component to dictate how to spawn players.
#[derive(Debug, Component)]
pub struct PlayerSpawner {
    pub abilities: Vec<Ability>,
}
