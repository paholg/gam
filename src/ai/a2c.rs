mod env;
mod model;

use bevy::prelude::{NonSendMut, Plugin, Query, Res, With};
use bevy_rapier2d::prelude::Velocity;

use crate::{Enemy, FixedTimestepSystem};

use self::{
    env::Team,
    model::{Sampler, Trainer},
};

use super::AiState;

pub struct A2CTrainerPlugin;
pub struct A2CSamplerPlugin;

struct EnemyTrainer {
    trainer: Trainer,
}

struct EnemySampler {
    sampler: Sampler,
}

impl Plugin for A2CTrainerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_non_send_resource(EnemyTrainer {
            trainer: Trainer::new(Team::Enemy),
        })
        .add_engine_tick_system(train_enemy);
    }
}

impl Plugin for A2CSamplerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_non_send_resource(EnemySampler {
            sampler: Sampler::new(Team::Enemy),
        })
        .add_engine_tick_system(sample_enemy);
    }
}

fn train_enemy(
    mut trainer: NonSendMut<EnemyTrainer>,
    ai_state: Res<AiState>,
    enemies: Query<&mut Velocity, With<Enemy>>,
) {
    trainer.trainer.train_one_step(&ai_state, enemies);
}

fn sample_enemy(
    mut sampler: NonSendMut<EnemySampler>,
    ai_state: Res<AiState>,
    enemies: Query<&mut Velocity, With<Enemy>>,
) {
    sampler.sampler.sample_one_step(&ai_state, enemies);
}
