use bevy::prelude::{Added, Commands, Component, Entity, Plugin, Query, Vec3, With};
use bevy_rapier3d::prelude::Velocity;
use rurel::{
    mdp::{Agent, State},
    strategy::{explore::RandomExploration, learn::QLearning, terminate::FixedIterations},
    AgentTrainer,
};

use crate::{Ai, Ally, Enemy, FixedTimestepSystem, Health, SPEED};

// For now, we will still use the simple ai that rotates and shoots, and use
// this just for movement.

pub struct QLearningPlugin;

impl Plugin for QLearningPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_engine_tick_system(add_ai_system)
            .add_engine_tick_system(train_enemy_ai_system);
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Default)]
struct AiState {
    dx: eq_float::F32,
    dy: eq_float::F32,
    friendly_health: eq_float::F64,
    enemy_health: eq_float::F64,
}

impl State for AiState {
    type A = AiAction;

    fn reward(&self) -> f64 {
        f64::from(self.friendly_health) - f64::from(self.enemy_health)
    }

    fn actions(&self) -> Vec<Self::A> {
        vec![
            AiAction {
                dx: 0.0.into(),
                dy: 0.0.into(),
            },
            AiAction {
                dx: SPEED.into(),
                dy: 0.0.into(),
            },
            AiAction {
                dx: (-SPEED).into(),
                dy: 0.0.into(),
            },
            AiAction {
                dx: 0.0.into(),
                dy: SPEED.into(),
            },
            AiAction {
                dx: 0.0.into(),
                dy: (-SPEED).into(),
            },
        ]
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct AiAction {
    dx: eq_float::F32,
    dy: eq_float::F32,
}

#[derive(Component)]
struct AiAgent {
    state: AiState,
}

impl Agent<AiState> for AiAgent {
    fn current_state(&self) -> &AiState {
        &self.state
    }

    fn take_action(&mut self, action: &<AiState as State>::A) {
        let AiAction { dx, dy } = action;
        self.state.dx = *dx;
        self.state.dy = *dy;
    }
}

#[derive(Component)]
struct AiTrainer {
    trainer: AgentTrainer<AiState>,
}

fn add_ai_system(mut commands: Commands, ai_query: Query<Entity, Added<Ai>>) {
    for entity in ai_query.iter() {
        let trainer = AiTrainer {
            trainer: AgentTrainer::new(),
        };
        commands.entity(entity).insert(trainer);
    }
}

fn train_enemy_ai_system(
    ally_query: Query<&Health, With<Ally>>,
    enemy_query: Query<&Health, With<Enemy>>,
    mut ai_query: Query<(&mut AiTrainer, &mut Velocity)>,
) {
    let ally_health: f32 = ally_query.iter().map(|health| health.cur).sum();
    let enemy_health: f32 = enemy_query.iter().map(|health| health.cur).sum();

    for (mut trainer, mut velocity) in ai_query.iter_mut() {
        let mut agent = AiAgent {
            state: AiState {
                friendly_health: (enemy_health as f64).into(),
                enemy_health: (ally_health as f64).into(),
                dx: 0.0.into(),
                dy: 0.0.into(),
            },
        };
        trainer.trainer.train(
            &mut agent,
            &QLearning::new(0.2, 0.01, 2.0),
            &mut FixedIterations::new(1),
            &RandomExploration::new(),
        );
        velocity.linvel = Vec3::new(agent.state.dx.into(), agent.state.dy.into(), 0.0);
    }
}
