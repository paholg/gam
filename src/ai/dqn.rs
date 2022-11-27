// use bevy::prelude::{default, Query, Vec2, With};
// use bevy_rapier2d::prelude::Velocity;
// use tch::{kind::FLOAT_CPU, nn, Tensor};
// use tracing::info;

// use crate::{ai::AiState, Enemy, SPEED};

// const NUMBER: i64 = 4;

// #[derive(Debug)]
// pub struct Step {
//     pub obs: Tensor,
//     pub reward: Tensor,
//     pub is_done: Tensor,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub enum Team {
//     Ally,
//     Enemy,
// }

// #[derive(Debug)]
// pub struct Env {
//     team: Team,
//     action_space: i64,
//     buf: Vec<f32>,
//     observation_space: Vec<i64>,
// }

// const ACTIONS: [Vec2; 5] = [
//     Vec2::new(0.0, 0.0),
//     Vec2::new(1.0, 0.0),
//     Vec2::new(-1.0, 0.0),
//     Vec2::new(0.0, 1.0),
//     Vec2::new(0.0, -1.0),
// ];

// impl Env {
//     const ACTION_SPACE: i64 = ACTIONS.len() as i64;
//     const OBSERVATION_SPACE: [i64; 1] = [4];

//     pub fn new(team: Team) -> Self {
//         // Number of possible actions?
//         let action_space = 5;

//         // The shape of the world???
//         let observation_space = vec![1, NUMBER];

//         Self {
//             team,
//             action_space,
//             buf: vec![0.0; NUMBER as usize],
//             observation_space,
//         }
//     }

//     /// Reset the state of the world???
//     /// And return `obs`, I think, being the world.
//     pub fn reset(&self) -> Tensor {
//         Tensor::of_slice(&self.buf)
//             .view_(&self.observation_space)
//             .to_kind(tch::Kind::Float)
//     }

//     // For now, let's always have 1 ally and 1 enemy.
//     pub fn step(
//         &mut self,
//         action: Vec<i64>,
//         ai_state: &AiState,
//         mut enemies: Query<&mut Velocity, With<Enemy>>,
//     ) -> Step {
//         let reward = if self.team == Team::Ally {
//             ai_state.ally_dmg_done - ai_state.enemy_dmg_done
//         } else {
//             ai_state.enemy_dmg_done - ai_state.ally_dmg_done
//         };

//         if let Ok(mut enemy) = enemies.get_single_mut() {
//             enemy.linvel = ACTIONS[action[0] as usize] * SPEED;
//         }

//         // This is bad. Worry about it later.
//         self.buf[0] = ai_state.ally_location.x;
//         self.buf[1] = ai_state.ally_location.y;
//         self.buf[2] = ai_state.enemy_location.x;
//         self.buf[3] = ai_state.enemy_location.y;

//         let obs = Tensor::of_slice(&self.buf)
//             .view_(&self.observation_space)
//             .to_kind(tch::Kind::Float);
//         Step {
//             obs,
//             reward: Tensor::from(reward),
//             is_done: Tensor::from(1.0f32),
//         }
//     }

//     pub fn action_space(&self) -> i64 {
//         self.action_space
//     }
// }

// struct ReplayBuffer {
//     buffer_size: usize,
//     pos: usize,
//     full: bool,
//     n_envs: usize,
//     optimize_memory_usage: bool,
//     handle_timeout_termination: bool,
// }

// impl ReplayBuffer {
//     // fn new(buffer_size: usize) -> Self {
//     //     let n_envs = 1;
//     //     let buffer_size = (buffer_size / n_envs).max(1);
//     //     let observations =
//     //     Self {
//     //         buffer_size,
//     //         pos: 0,
//     //         full: false,
//     //         n_envs,

//     //         // Note: These can't both be true.
//     //         optimize_memory_usage: false,
//     //         handle_timeout_termination: true,
//     //     }
//     // }

//     fn add() {}

//     fn sample() {}
// }

// type QNetwork = Box<dyn Fn(&Tensor) -> Tensor>;

// fn q_network(p: &nn::Path) -> QNetwork {
//     let seq = nn::seq()
//         .add(nn::linear(
//             p / "lin1",
//             Env::OBSERVATION_SPACE.into_iter().product(),
//             120,
//             nn::LinearConfig::default(),
//         ))
//         .add_fn(Tensor::relu)
//         .add(nn::linear(p / "lin2", 120, 84, nn::LinearConfig::default()))
//         .add_fn(Tensor::relu)
//         .add(nn::linear(
//             p / "lin3",
//             84,
//             Env::ACTION_SPACE,
//             nn::LinearConfig::default(),
//         ));

//     let device = p.device();
//     Box::new(move |xs: &Tensor| xs.to_device(device).apply(&seq))
// }

// fn linear_schedule(start_e: f32, end_e: f32, duration: i64, t: i64) -> f32 {
//     let slope = (end_e - start_e) / duration as f32;
//     (slope * t as f32 + start_e).max(end_e)
// }

// pub struct Trainer {
//     q_network: QNetwork,
//     optimizer: nn::Optimizer,
//     target_network: QNetwork,
//     replay_buffer: ReplayBuffer,
// }
