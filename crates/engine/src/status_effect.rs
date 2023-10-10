use std::collections::HashSet;

use bevy_ecs::component::Component;

#[derive(Component, Default)]
pub struct StatusEffects {
    pub effects: HashSet<StatusEffect>,
}

#[derive(Hash, PartialEq, Eq)]
pub enum StatusEffect {
    HyperSprinting,
}
