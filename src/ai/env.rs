use bevy::prelude::{Query, Vec2, With};
use bevy_rapier2d::prelude::Velocity;
use tch::{kind::FLOAT_CPU, Tensor};
use tracing::info;

use crate::{ai::AiState, Enemy, PLANE_SIZE, SPEED};

use super::{COLS, ROWS};

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
    observation_space: Tensor,
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

        // The shape of the world???
        let observation_space = Tensor::zeros(&[ROWS, COLS], FLOAT_CPU);

        Self {
            team,
            action_space,
            observation_space,
        }
    }

    /// Reset the state of the world???
    /// And return `obs`, I think, being the world.
    pub fn reset(&self) -> Tensor {
        self.observation_space.detach()
    }

    // For now, let's always have 1 ally and 1 enemy.
    pub fn step(
        &mut self,
        action: Vec<i64>,
        ai_state: &AiState,
        mut enemies: Query<&mut Velocity, With<Enemy>>,
    ) -> Step {
        let reward = if self.team == Team::Ally {
            ai_state.ally_dmg_done - ai_state.enemy_dmg_done
        } else {
            ai_state.enemy_dmg_done - ai_state.ally_dmg_done
        };

        if let Ok(mut enemy) = enemies.get_single_mut() {
            enemy.linvel = ACTIONS[action[0] as usize] * SPEED;
        }

        let rows = Tensor::of_slice(&[0i64, 1, 0, 1]);
        let columns = Tensor::of_slice(&[0i64, 0, 1, 1]);
        let values = Tensor::of_slice(&[
            ai_state.ally_location.x / PLANE_SIZE * 2.0,
            ai_state.ally_location.y / PLANE_SIZE * 2.0,
            ai_state.enemy_location.x / PLANE_SIZE * 2.0,
            ai_state.enemy_location.y / PLANE_SIZE * 2.0,
        ]);
        let obs = self
            .observation_space
            .index_put(&[Some(rows), Some(columns)], &values, false);

        Step {
            obs,
            reward: Tensor::from(reward),
            is_done: Tensor::from(1.0f32),
        }
    }

    pub fn action_space(&self) -> i64 {
        self.action_space
    }
}
