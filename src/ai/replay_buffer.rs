use std::collections::HashMap;

use tch::{kind::FLOAT_CPU, nn, Tensor};
use typed_builder::TypedBuilder;

pub struct ReplayBufferSamples {
    pub observations: Tensor,
    pub actions: Tensor,
    pub old_values: Tensor,
    pub old_log_porb: Tensor,
    pub advantages: Tensor,
    pub returns: Tensor,
}

// See python's stable_baselines3 ReplayBuffer
#[derive(TypedBuilder)]
pub struct ReplayBuffer<'a> {
    path: nn::Path<'a>,
    buffer_size: i64,

    // TODO: This shouldn't be a vec, but more flexible.
    observation_space: Vec<f32>,
    action_space: Vec<i64>,

    #[builder(default = 1.0)]
    gae_lambda: f32,
    #[builder(default = 0.99)]
    gamma: f32,
    #[builder(default = 1)]
    n_envs: i64,

    #[builder(default = false)]
    optimize_memory_usage: bool,
    #[builder(default = true)]
    handle_timeout_termination: bool,

    #[builder(default, setter(skip))]
    observations: Tensor,
    #[builder(default, setter(skip))]
    next_observations: Tensor,
    #[builder(default, setter(skip))]
    actions: Tensor,
    #[builder(default, setter(skip))]
    rewards: Tensor,
    #[builder(default, setter(skip))]
    dones: Tensor,
    #[builder(default, setter(skip))]
    timeouts: Tensor,
}

impl<'a> ReplayBuffer<'a> {
    /// Note: Call on a newly constructed ReplayBuffer.
    // todo: make the first call not necessary.
    pub fn init(&mut self) {
        self.buffer_size = (self.buffer_size / self.n_envs).max(1);

        if self.optimize_memory_usage && self.handle_timeout_termination {
            panic!("Replyay buffer does not support both optimize_memory_usage and handle_timeout_termination");
        }

        self.observations = Tensor::zeros(
            &[self.buffer_size, self.n_envs, self.obs_shape()],
            FLOAT_CPU,
        );

        if !self.optimize_memory_usage {
            self.next_observations = Tensor::zeros(
                &[self.buffer_size, self.n_envs, self.obs_shape()],
                FLOAT_CPU,
            );
        }

        self.actions = Tensor::zeros(
            &[self.buffer_size, self.n_envs, self.action_dim()],
            FLOAT_CPU,
        );
        self.rewards = Tensor::zeros(&[self.buffer_size, self.n_envs], FLOAT_CPU);
        self.dones = Tensor::zeros(&[self.buffer_size, self.n_envs], FLOAT_CPU);

        self.timeouts = Tensor::zeros(&[self.buffer_size, self.n_envs], FLOAT_CPU);
    }

    pub fn add(
        &mut self,
        obs: Tensor,
        next_obs: Tensor,
        action: Tensor,
        reward: Tensor,
        done: Tensor,
        infos: Vec<HashMap<String, ()>>,
    ) {
    }

    pub fn sample(&self) -> ReplayBufferSamples {
        todo!()
    }

    fn obs_shape(&self) -> i64 {
        self.observation_space.len() as i64
    }

    fn action_dim(&self) -> i64 {
        self.action_space.len() as i64
    }
}
