use bevy::prelude::{Plugin, Query, ResMut, Resource, Transform, Vec2, With, Without};

use crate::{Ally, Enemy, FixedTimestepSystem};

pub mod a2c;
pub mod f32;
// pub mod qlearning;
pub mod simple;

#[derive(Resource, Default)]
pub struct AiState {
    pub ally_dmg_done: f32,
    pub enemy_dmg_done: f32,

    pub ally_location: Vec2,
    pub enemy_location: Vec2,
}

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(AiState::default())
            .add_engine_tick_system(ai_state_system);
    }
}

fn ai_state_system(
    ally: Query<&Transform, (With<Ally>, Without<Enemy>)>,
    enemy: Query<&Transform, (With<Enemy>, Without<Ally>)>,
    mut ai_state: ResMut<AiState>,
) {
    let ally = ally.get_single();
    let enemy = enemy.get_single();

    if let Ok(ally) = ally {
        ai_state.ally_location = ally.translation.truncate();
    }
    if let Ok(enemy) = enemy {
        ai_state.enemy_location = enemy.translation.truncate();
    }
}
