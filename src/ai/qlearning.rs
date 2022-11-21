use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
    thread::{self, Thread},
};

use bevy::prelude::{Plugin, Query, Res, ResMut, Resource, Transform, Vec2, With, Without};
use bevy_rapier2d::prelude::Velocity;
use rurel::{
    mdp::{Agent, State},
    strategy::{explore::RandomExploration, learn::QLearning, terminate::FixedIterations},
    AgentTrainer,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{config::project_dirs, time::TickCounter, Ai, Ally, Enemy, FixedTimestepSystem, SPEED};

use super::f32::F32;

// For now, we will still use the simple ai that rotates and shoots, and use
// this just for movement.

pub struct QLearningPlugin;

impl Plugin for QLearningPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let enemy_thread = thread::spawn(|| EnemyAi::default());
        let ally_thread = thread::spawn(|| AllyAi::default());
        app.insert_resource(enemy_thread.join().unwrap())
            .insert_resource(ally_thread.join().unwrap())
            .add_engine_tick_system(train_ai_system);
    }
}

fn ally_path() -> PathBuf {
    let project_dirs = project_dirs().unwrap();
    let cache_dir = project_dirs.cache_dir();

    fs::create_dir_all(cache_dir).unwrap();

    let mut path = cache_dir.to_owned();
    path.push("ally_qlearning.bin");
    path
}

fn enemy_path() -> PathBuf {
    let project_dirs = project_dirs().unwrap();
    let cache_dir = project_dirs.cache_dir();

    fs::create_dir_all(cache_dir).unwrap();

    let mut path = cache_dir.to_owned();
    path.push("enemy_qlearning.bin");
    path
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vec2Eq {
    x: F32,
    y: F32,
}

impl From<&Transform> for Vec2Eq {
    fn from(transform: &Transform) -> Self {
        Self {
            x: transform.translation.x.into(),
            y: transform.translation.y.into(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Default, Serialize, Deserialize)]
pub struct AiState {
    // dv really shouldn't be here, as it's an output only, but I see no other
    // way to access it.
    dv: Vec2Eq,
    location: Vec2Eq,
    friendly_locations: Vec<Vec2Eq>,
    unfriendly_locations: Vec<Vec2Eq>,
    damage_done: F32,
    damage_received: F32,
}

impl State for AiState {
    type A = AiAction;

    fn reward(&self) -> f64 {
        (f32::from(self.damage_done) - f32::from(self.damage_received)) as f64
    }

    fn actions(&self) -> Vec<Self::A> {
        vec![
            AiAction {
                dir: Vec2Eq {
                    x: 0.0.into(),
                    y: 0.0.into(),
                },
            },
            AiAction {
                dir: Vec2Eq {
                    x: SPEED.into(),
                    y: 0.0.into(),
                },
            },
            AiAction {
                dir: Vec2Eq {
                    x: (-SPEED).into(),
                    y: 0.0.into(),
                },
            },
            AiAction {
                dir: Vec2Eq {
                    x: 0.0.into(),
                    y: SPEED.into(),
                },
            },
            AiAction {
                dir: Vec2Eq {
                    x: 0.0.into(),
                    y: (-SPEED).into(),
                },
            },
        ]
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AiAction {
    dir: Vec2Eq,
}

#[derive(Default)]
pub struct AiAgent {
    state: AiState,
}

impl Agent<AiState> for AiAgent {
    fn current_state(&self) -> &AiState {
        &self.state
    }

    fn take_action(&mut self, action: &<AiState as State>::A) {
        self.state.dv = action.dir;
    }
}

fn load_ai_data(path: &Path) -> Result<HashMap<AiState, HashMap<AiAction, f64>>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    info!("Deserializing...");
    let state = bincode::serde::decode_from_reader(reader, bincode::config::standard())
        .map_err(|e| io::Error::other(e))?;
    Ok(state)
}

fn save_ai_data(
    path: &Path,
    data: &HashMap<AiState, HashMap<AiAction, f64>>,
) -> Result<(), io::Error> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    info!("Serializing...");
    bincode::serde::encode_into_std_write(data, &mut writer, bincode::config::standard())
        .map_err(|e| io::Error::other(e))?;
    Ok(())
}

// We wrap the fields in Option so we can mutable borrow them both at the same
// time.
#[derive(Resource)]
pub struct EnemyAi {
    trainer: Option<AgentTrainer<AiState>>,
    agent: Option<AiAgent>,
}

impl Default for EnemyAi {
    fn default() -> Self {
        info!("Loading enemy data...");
        let mut trainer = AgentTrainer::default();
        if let Ok(state) = load_ai_data(&enemy_path()) {
            info!("Importing...");
            trainer.import_state(state);
            info!("Done loading");
        } else {
            warn!("Failed to load");
        }
        Self {
            trainer: Some(trainer),
            agent: Some(AiAgent::default()),
        }
    }
}

impl EnemyAi {
    pub fn take_damage(&mut self, damage: f32) {
        let state = &mut self.agent.as_mut().unwrap().state;
        state.damage_received = (f32::from(state.damage_received) + damage).into();
    }

    pub fn do_damage(&mut self, damage: f32) {
        let state = &mut self.agent.as_mut().unwrap().state;
        state.damage_done = (f32::from(state.damage_done) + damage).into();
    }
}

// We wrap the fields in Option so we can mutable borrow them both at the same
// time.
#[derive(Resource)]
pub struct AllyAi {
    trainer: Option<AgentTrainer<AiState>>,
    agent: Option<AiAgent>,
}

impl Default for AllyAi {
    fn default() -> Self {
        info!("Loading ally data...");
        let mut trainer = AgentTrainer::default();
        if let Ok(state) = load_ai_data(&ally_path()) {
            info!("Importing...");
            trainer.import_state(state);
            info!("Done loading");
        } else {
            warn!("Failed to load");
        }
        Self {
            trainer: Some(trainer),
            agent: Some(AiAgent::default()),
        }
    }
}

impl AllyAi {
    pub fn take_damage(&mut self, damage: f32) {
        let state = &mut self.agent.as_mut().unwrap().state;
        state.damage_received = (f32::from(state.damage_received) + damage).into();
    }

    pub fn do_damage(&mut self, damage: f32) {
        let state = &mut self.agent.as_mut().unwrap().state;
        state.damage_done = (f32::from(state.damage_done) + damage).into();
    }
}

// TODO: training should only happen in headless mode.
fn train_ai_system(
    mut ally_query: Query<(&Transform, &mut Velocity), (With<Ally>, Without<Enemy>, With<Ai>)>,
    mut enemy_query: Query<(&Transform, &mut Velocity), (With<Enemy>, Without<Ally>, With<Ai>)>,
    mut ally_ai: ResMut<AllyAi>,
    mut enemy_ai: ResMut<EnemyAi>,
    tick: Res<TickCounter>,
) {
    let ally_state = &mut ally_ai.agent.as_mut().unwrap().state;
    let enemy_state = &mut enemy_ai.agent.as_mut().unwrap().state;
    // Update locations:
    ally_state.friendly_locations.clear();
    ally_state.unfriendly_locations.clear();
    enemy_state.friendly_locations.clear();
    enemy_state.unfriendly_locations.clear();

    for ally in ally_query.iter().map(|(transform, _)| transform.into()) {
        ally_state.friendly_locations.push(ally);
        enemy_state.unfriendly_locations.push(ally);
    }

    for enemy in enemy_query.iter().map(|(transform, _)| transform.into()) {
        ally_state.friendly_locations.push(enemy);
        enemy_state.unfriendly_locations.push(enemy);
    }

    let mut ally_agent = ally_ai.agent.take().unwrap();
    let mut ally_trainer = ally_ai.trainer.take().unwrap();
    for (transform, mut velocity) in ally_query.iter_mut() {
        ally_agent.state.location = transform.into();
        ally_trainer.train(
            &mut ally_agent,
            &QLearning::new(0.2, 0.01, 2.0),
            &mut FixedIterations::new(1),
            &RandomExploration::new(),
        );
        velocity.linvel = Vec2::new(ally_agent.state.dv.x.into(), ally_agent.state.dv.y.into());
    }
    ally_ai.agent = Some(ally_agent);
    ally_ai.trainer = Some(ally_trainer);

    let mut enemy_agent = enemy_ai.agent.take().unwrap();
    let mut enemy_trainer = enemy_ai.trainer.take().unwrap();
    for (transform, mut velocity) in enemy_query.iter_mut() {
        enemy_agent.state.location = transform.into();
        enemy_trainer.train(
            &mut enemy_agent,
            &QLearning::new(0.2, 0.01, 2.0),
            &mut FixedIterations::new(1),
            &RandomExploration::new(),
        );
        velocity.linvel = Vec2::new(enemy_agent.state.dv.x.into(), enemy_agent.state.dv.y.into());
    }
    enemy_ai.agent = Some(enemy_agent);
    enemy_ai.trainer = Some(enemy_trainer);

    if tick.should_save() {
        let ally_dmg = ally_ai.agent.as_ref().unwrap().state.damage_done;
        let enemy_dmg = enemy_ai.agent.as_ref().unwrap().state.damage_done;
        info!(%ally_dmg, %enemy_dmg);

        thread::scope(|scope| {
            let t1 = scope.spawn(|| {
                info!("Saving ally...");
                // TODO: export_learned_values clones needlessly.
                let ally_values = ally_ai.trainer.as_ref().unwrap().learned_values();
                save_ai_data(&ally_path(), ally_values).unwrap();
            });

            let t2 = scope.spawn(|| {
                info!("Saving enemy...");
                let enemy_values = enemy_ai.trainer.as_ref().unwrap().learned_values();
                save_ai_data(&enemy_path(), enemy_values).unwrap();
            });
            t1.join().unwrap();
            t2.join().unwrap();
            info!("Done saving");
        });
    }
}
