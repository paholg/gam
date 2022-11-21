use std::{ops::Mul, time::Duration};

use bevy::{
    prelude::{Plugin, Res, ResMut, Resource},
    time::Time,
};
use tracing::info;

use crate::FixedTimestepSystem;

pub const TIMESTEP: Duration = Duration::from_secs_f64(1.0 / 60.0);
pub const ENGINE_TICK: &str = "engine_tick";

/// Represents a duration in ticks rather than time.
#[derive(Default)]
pub struct Tick {
    val: u64,
}

impl Tick {
    /// Construct a new `Tick` from a duration using the engine `TIMESTEP`.
    pub const fn new(duration: Duration) -> Self {
        // This function is a bit funky as we're limited by what we can do in a
        // const function. E.g. No access to `round` or `max`.
        let ticks = (duration.div_duration_f64(TIMESTEP) + 0.5) as u64;

        let val = if ticks == 0 { 1 } else { ticks };

        Self { val }
    }

    pub fn tick(&mut self) -> &Self {
        self.val = self.val.saturating_sub(1);
        self
    }

    pub fn is_zero(&self) -> bool {
        self.val == 0
    }
}

impl Mul<u64> for Tick {
    type Output = Tick;

    fn mul(self, rhs: u64) -> Self::Output {
        Self {
            val: self.val * rhs,
        }
    }
}

#[derive(Resource, Default)]
struct TickCounter {
    tick: Tick,
}

fn log_tick_system(mut tick_counter: ResMut<TickCounter>, time: Res<Time>) {
    tick_counter.tick.val += 1;
    let tick = tick_counter.tick.val;
    let delta = time.delta_seconds();

    info!(tick, delta, "Tick");
}

pub struct TickDebugPlugin;

impl Plugin for TickDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TickCounter::default())
            .add_engine_tick_system(log_tick_system);
    }
}
