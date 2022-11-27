use bevy::prelude::{Plugin, Query, ResMut, Resource, Transform, Vec2, With, Without};
use tch::{kind::FLOAT_CPU, Tensor};

use crate::{Ally, Enemy, FixedTimestepSystem};

pub mod a2c;
pub mod f32;
// pub mod qlearning;
pub mod dqn;
pub mod env;
pub mod ppo;
pub mod replay_buffer;
pub mod simple;

pub const ROWS: i64 = 84;
pub const COLS: i64 = 84;

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

pub struct FrameStack {
    data: Tensor,
    nprocs: i64,
    nstack: i64,
}

impl FrameStack {
    pub fn new(nprocs: i64, nstack: i64) -> FrameStack {
        FrameStack {
            data: Tensor::zeros(&[nprocs, nstack, ROWS, COLS], FLOAT_CPU),
            nprocs,
            nstack,
        }
    }

    pub fn update<'a>(&'a mut self, img: &Tensor, masks: Option<&Tensor>) -> &'a Tensor {
        if let Some(masks) = masks {
            self.data *= masks.view([self.nprocs, 1, 1, 1])
        };
        let slice = |i| self.data.narrow(1, i, 1);
        for i in 1..self.nstack {
            slice(i - 1).copy_(&slice(i))
        }
        slice(self.nstack - 1).copy_(img);
        &self.data
    }
}
