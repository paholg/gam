use bevy::prelude::{default, Query, With};
use bevy_rapier2d::prelude::Velocity;
use tch::{
    kind::{FLOAT_CPU, INT64_CPU},
    nn::{self, OptimizerConfig},
    Device, Kind, Tensor,
};
use tracing::{info, warn};

use crate::{
    ai::{
        env::{Env, Team},
        AiState, FrameStack, COLS, ROWS,
    },
    Enemy,
};

const NPROCS: i64 = 1; // 16;
const NSTEPS: i64 = 5;
const NSTACK: i64 = 4;

const LEARNING_RATE: f64 = 1e-4;

type Model = Box<dyn Fn(&Tensor) -> (Tensor, Tensor)>;

pub fn model(p: &nn::Path, n_actions: i64) -> Model {
    let stride = |stride| nn::ConvConfig {
        stride,
        ..default()
    };

    let seq = nn::seq()
        .add(nn::conv2d(p / "c1", NSTACK, 32, 8, stride(4)))
        .add_fn(|xs| xs.relu())
        .add(nn::conv2d(p / "c2", 32, 64, 4, stride(2)))
        .add_fn(|xs| xs.relu())
        .add(nn::conv2d(p / "c3", 64, 64, 3, stride(1)))
        .add_fn(|xs| xs.relu().flat_view())
        .add(nn::linear(p / "l1", 3136, 512, Default::default()))
        .add_fn(|xs| xs.relu());
    let critic = nn::linear(p / "cl", 512, 1, Default::default());
    let actor = nn::linear(p / "al", 512, n_actions, Default::default());
    let device = p.device();
    Box::new(move |xs: &Tensor| {
        let xs = xs.to_device(device).apply(&seq);
        (xs.apply(&critic), xs.apply(&actor))
    })
}
pub struct Trainer {
    env: Env,
    device: Device,
    vs: nn::VarStore,
    model: Model,
    opt: nn::Optimizer,

    sum_rewards: Tensor,
    total_rewards: f64,
    total_episodes: f64,

    frame_stack: FrameStack,

    s_states: Tensor,

    update_index: i64,
    step_index: i64,

    update_data: UpdateData,
}

struct UpdateData {
    s_values: Tensor,
    s_rewards: Tensor,
    s_actions: Tensor,
    s_masks: Tensor,
}

impl UpdateData {
    fn new() -> Self {
        Self {
            s_values: Tensor::zeros(&[NSTEPS, NPROCS], FLOAT_CPU),
            s_rewards: Tensor::zeros(&[NSTEPS, NPROCS], FLOAT_CPU),
            s_actions: Tensor::zeros(&[NSTEPS, NPROCS], INT64_CPU),
            s_masks: Tensor::zeros(&[NSTEPS, NPROCS], FLOAT_CPU),
        }
    }
}

impl Trainer {
    pub fn new(team: Team) -> Self {
        let env = Env::new(team);
        info!(
            action_space = %env.action_space(),
            "Initializing A2C Trainer"
        );

        let device = tch::Device::cuda_if_available();
        let mut vs = nn::VarStore::new(device);
        let model = model(&vs.root(), env.action_space());
        if let Err(error) = vs.load("a2c.ot") {
            warn!(%error, "Error loading AI model");
        }
        let opt = nn::Adam::default().build(&vs, LEARNING_RATE).unwrap();

        let sum_rewards = Tensor::zeros(&[NPROCS], FLOAT_CPU);
        let total_rewards = 0.0;
        let total_episodes = 0.0;

        let mut frame_stack = FrameStack::new(NPROCS, NSTACK);
        let _ = frame_stack.update(&env.reset(), None);
        let s_states = Tensor::zeros(&[NSTEPS + 1, NPROCS, NSTACK, ROWS, COLS], FLOAT_CPU);
        s_states.get(0).copy_(&s_states.get(-1));

        Self {
            env,
            device,
            vs,
            model,
            opt,
            sum_rewards,
            total_rewards,
            total_episodes,
            frame_stack,
            s_states,
            update_index: 0,
            step_index: 0,

            update_data: UpdateData::new(),
        }
    }

    pub fn train_one_step(
        &mut self,
        ai_state: &AiState,
        enemies: Query<&mut Velocity, With<Enemy>>,
    ) {
        let (critic, actor) = tch::no_grad(|| (self.model)(&self.s_states.get(self.step_index)));
        let probs = actor.softmax(-1, Kind::Float);
        let actions = probs.multinomial(1, true).squeeze_dim(-1);
        let step = self
            .env
            .step(Vec::<i64>::from(&actions), &ai_state, enemies);

        self.sum_rewards += &step.reward;
        self.total_rewards += f64::from((&self.sum_rewards * &step.is_done).sum(Kind::Float));
        self.total_episodes += f64::from(step.is_done.sum(Kind::Float));

        let masks = Tensor::from(1f32) - step.is_done;
        self.sum_rewards *= &masks;
        let obs = self.frame_stack.update(&step.obs, Some(&masks));
        self.update_data
            .s_actions
            .get(self.step_index)
            .copy_(&actions);
        self.update_data
            .s_values
            .get(self.step_index)
            .copy_(&critic.squeeze_dim(-1));
        self.s_states.get(self.step_index + 1).copy_(obs);
        self.update_data
            .s_rewards
            .get(self.step_index)
            .copy_(&step.reward);
        self.update_data.s_masks.get(self.step_index).copy_(&masks);

        self.step_index += 1;

        if self.step_index % NSTEPS == 0 {
            self.step_index = 0;

            let s_returns = {
                let r = Tensor::zeros(&[NSTEPS + 1, NPROCS], FLOAT_CPU);
                let critic = tch::no_grad(|| (self.model)(&self.s_states.get(-1)).0);
                r.get(-1).copy_(&critic.view([NPROCS]));
                for s in (0..NSTEPS).rev() {
                    let r_s = self.update_data.s_rewards.get(s)
                        + r.get(s + 1) * self.update_data.s_masks.get(s) * 0.99;
                    r.get(s).copy_(&r_s);
                }
                r
            };
            let (critic, actor) = (self.model)(&self.s_states.narrow(0, 0, NSTEPS).view([
                NSTEPS * NPROCS,
                NSTACK,
                ROWS,
                COLS,
            ]));
            let critic = critic.view([NSTEPS, NPROCS]);
            let actor = actor.view([NSTEPS, NPROCS, -1]);
            let log_probs = actor.log_softmax(-1, Kind::Float);
            let probs = actor.softmax(-1, Kind::Float);
            let action_log_probs = {
                let index = self
                    .update_data
                    .s_actions
                    .unsqueeze(-1)
                    .to_device(self.device);
                log_probs.gather(2, &index, false).squeeze_dim(-1)
            };
            let dist_entropy = (-log_probs * probs)
                .sum_dim_intlist(Some([-1].as_slice()), false, Kind::Float)
                .mean(Kind::Float);
            let advantages = s_returns.narrow(0, 0, NSTEPS).to_device(self.device) - critic;
            let value_loss = (&advantages * &advantages).mean(Kind::Float);
            let action_loss = (-advantages.detach() * action_log_probs).mean(Kind::Float);
            let loss = value_loss * 0.5 + action_loss - dist_entropy * 0.01;
            self.opt.backward_step_clip(&loss, 0.5);
            if self.update_index > 0 && self.update_index % 500 == 0 {
                info!(
                    %self.update_index,
                    %self.total_rewards,

                    "Ai tick",
                );
                self.total_rewards = 0.;
                self.total_episodes = 0.;
            }
            if self.update_index > 0 && self.update_index % 10000 == 0 {
                if let Err(error) = self.vs.save("a2c.ot") {
                    warn!(%error, "Error while saving model data");
                } else {
                    info!("Saved model data!");
                }
            }

            self.s_states.get(0).copy_(&self.s_states.get(-1));
            self.update_data.s_values = Tensor::zeros(&[NSTEPS, NPROCS], FLOAT_CPU);
            self.update_data.s_rewards = Tensor::zeros(&[NSTEPS, NPROCS], FLOAT_CPU);
            self.update_data.s_actions = Tensor::zeros(&[NSTEPS, NPROCS], INT64_CPU);
            self.update_data.s_masks = Tensor::zeros(&[NSTEPS, NPROCS], FLOAT_CPU);

            self.update_index += 1;
        }
    }
}

pub struct Sampler {
    env: Env,
    frame_stack: FrameStack,
    obs: Tensor,
    model: Model,
}

impl Sampler {
    pub fn new(team: Team) -> Self {
        let env = Env::new(team);
        let mut vs = nn::VarStore::new(tch::Device::Cpu);
        let model = model(&vs.root(), env.action_space());
        if let Err(error) = vs.load("a2c.ot") {
            warn!(%error, "Error loading AI model");
        }

        let mut frame_stack = FrameStack::new(1, NSTACK);
        // fixme: detach :(
        let obs = frame_stack.update(&env.reset(), None).detach();

        Self {
            env,
            frame_stack,
            obs,
            model,
        }
    }

    pub fn sample_one_step(
        &mut self,
        ai_state: &AiState,
        enemies: Query<&mut Velocity, With<Enemy>>,
    ) {
        let (_critic, actor) = tch::no_grad(|| (self.model)(&self.obs));
        let probs = actor.softmax(-1, Kind::Float);
        let actions = probs.multinomial(1, true).squeeze_dim(-1);
        let step = self.env.step(Vec::<i64>::from(&actions), ai_state, enemies);
        let masks = Tensor::from(1f32) - step.is_done;
        self.obs = self.frame_stack.update(&step.obs, Some(&masks)).detach();
    }
}
