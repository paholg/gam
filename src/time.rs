use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use bevy::prelude::{Plugin, Res, ResMut, Resource};
use iyes_loopless::prelude::FixedTimesteps;
use tracing::info;

use crate::FixedTimestepSystem;

/// The timestep at which we run our game.
pub const TIMESTEP: Duration = Duration::from_secs_f32(PHYSICS_TIMESTEP);
/// The timestep the physics engine sees.
pub const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0;

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

        // TODO: Account for PHYSICS_TIMESTEP and TIMESTEP being different.
        // Things should have a duration that is sped up as we speed up the game.
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

#[derive(Resource)]
struct TickCounter {
    tick: Tick,
    since: Instant,
}

impl TickCounter {
    fn new() -> Self {
        Self {
            tick: Tick { val: 0 },
            since: Instant::now(),
        }
    }
}

fn debug_tick_system(mut tick_counter: ResMut<TickCounter>, timesteps: Res<FixedTimesteps>) {
    tick_counter.tick.val += 1;
    let tick = tick_counter.tick.val;

    let now = Instant::now();
    let dur = now.duration_since(tick_counter.since);
    tick_counter.since = now;

    let info = timesteps.get_current().unwrap();
    let dur_iyes = info.timestep();

    info!(tick, ?dur, ?dur_iyes, "Tick");
}

pub struct TickDebugPlugin;

impl Plugin for TickDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TickCounter::new())
            .add_engine_tick_system(debug_tick_system);
    }
}
