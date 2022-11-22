use bevy::prelude::Vec2;
use tch::{kind::FLOAT_CPU, Tensor};
use tracing::info;

use crate::ai::AiState;

#[derive(Debug)]
pub struct Step {
    pub obs: Tensor,
    pub reward: Tensor,
    pub is_done: Tensor,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Team {
    Ally,
    Enemy,
}

#[derive(Debug)]
pub struct Env {
    team: Team,
    action_space: i64,
    observation_space: Vec<i64>,
}

const ACTIONS: [Vec2; 5] = [
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 0.0),
    Vec2::new(-1.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(0.0, -1.0),
];

impl Env {
    pub fn new(team: Team) -> Self {
        // Number of possible actions?
        let action_space = 5;

        // The world???
        let observation_space = Vec::new();

        Self {
            team,
            action_space,
            observation_space,
        }
    }

    /// Reset the state of the world???
    /// And return `obs`, I think, being the world.
    pub fn reset(&self) -> Tensor {
        Tensor::zeros(&[4], FLOAT_CPU)
    }

    // For now, let's always have 1 ally and 1 enemy.
    pub fn step(&self, action: Vec<i64>, ai_state: &AiState) -> Step {
        info!(?action, "Step");

        let reward = if self.team == Team::Ally {
            ai_state.ally_dmg_done - ai_state.enemy_dmg_done
        } else {
            ai_state.enemy_dmg_done - ai_state.ally_dmg_done
        };

        // TODO: We're not actually moving!

        Step {
            obs: Tensor::of_slice(&[
                ai_state.ally_location.x,
                ai_state.ally_location.y,
                ai_state.enemy_location.x,
                ai_state.enemy_location.y,
            ]),
            reward: Tensor::from(reward),
            is_done: Tensor::from(0.0f32),
        }
    }

    pub fn action_space(&self) -> i64 {
        self.action_space
    }

    pub fn observation_space(&self) -> &[i64] {
        &self.observation_space
    }
}
