mod env;
mod model;

use bevy::prelude::{NonSendMut, Plugin, Query, Res, With};
use bevy_rapier2d::prelude::Velocity;

use crate::{Enemy, FixedTimestepSystem};

use self::{env::Team, model::Trainer};

use super::AiState;

pub struct A2CPlugin;

struct EnemyTrainer {
    trainer: Trainer,
}

impl Plugin for A2CPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_non_send_resource(EnemyTrainer {
            trainer: Trainer::new(Team::Enemy).unwrap(),
        })
        .add_engine_tick_system(run_enemy);
    }
}

fn run_enemy(
    mut trainer: NonSendMut<EnemyTrainer>,
    ai_state: Res<AiState>,
    enemies: Query<&mut Velocity, With<Enemy>>,
) {
    trainer.trainer.train_one_step(&ai_state, enemies).unwrap();
}
