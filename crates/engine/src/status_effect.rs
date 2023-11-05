use bevy_ecs::component::Component;
use bevy_reflect::Reflect;
use bevy_utils::HashSet;

#[derive(Component, Default, Reflect)]
pub struct StatusEffects {
    pub effects: HashSet<StatusEffect>,
}

#[derive(PartialEq, Eq, Hash, Reflect, Clone, Copy)]
pub enum StatusEffect {
    HyperSprinting,
}
