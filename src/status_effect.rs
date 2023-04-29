use std::collections::HashSet;

use bevy::prelude::Component;

#[derive(Component, Default)]
pub struct StatusEffects {
    pub effects: HashSet<StatusEffect>,
}

#[derive(Hash, PartialEq, Eq)]
pub enum StatusEffect {
    HyperSprinting,
}
