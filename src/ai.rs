use bevy::prelude::{Plugin, Query, ResMut, Resource, Transform, Vec2, With, Without};
use bevy_learn::{
    reinforce::{ReinforceConfig, ReinforceTrainer},
    Env,
};
use tch::{kind::FLOAT_CPU, Device, Tensor};

use crate::{Ally, Enemy, FixedTimestepSystem};

use self::env::{ai_act, ACTIONS};

pub mod env;
pub mod simple;

pub enum Action {
    Nothing,
    Move(Vec2),
}

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
        let initial_obs = vec![0.0; 6];
        let trainer = ReinforceTrainer::new(
            ReinforceConfig::builder().build(),
            Env::new(ACTIONS.len() as i64, 6, Device::cuda_if_available()),
            &initial_obs,
        );
        app.insert_resource(AiState::default())
            .insert_non_send_resource(trainer)
            .add_engine_tick_system(ai_act)
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
    pub fn new(nprocs: i64, nstack: i64, rows: i64, cols: i64) -> FrameStack {
        FrameStack {
            data: Tensor::zeros(&[nprocs, nstack, rows, cols], FLOAT_CPU),
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
