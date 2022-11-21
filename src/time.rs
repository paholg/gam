use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use bevy::prelude::{Plugin, Res, ResMut, Resource};
use iyes_loopless::prelude::FixedTimesteps;
use tracing::info;

use crate::FixedTimestepSystem;

/// The timestep at which we run our game.
/// When running graphically, this should always be the same as PHYSICS_TIMESTEP.
#[cfg(feature = "graphics")]
pub const TIMESTEP: Duration = Duration::from_secs_f32(PHYSICS_TIMESTEP);
#[cfg(not(feature = "graphics"))]
pub const TIMESTEP: Duration = Duration::from_nanos(1);
/// The timestep the physics engine sees.
pub const PHYSICS_INVERSE_TIMESTEP: f32 = 60.0;
pub const PHYSICS_TIMESTEP: f32 = 1.0 / PHYSICS_INVERSE_TIMESTEP;

/// Represents a duration in ticks rather than time.
#[derive(Default)]
pub struct Tick {
    val: u32,
}

impl Tick {
    /// Construct a new `Tick` from a duration using the engine `TIMESTEP`.
    pub const fn new(duration: Duration) -> Self {
        // This function is a bit funky as we're limited by what we can do in a
        // const function. E.g. No access to `round` or `max`.
        let ticks = (duration.as_secs_f32() * PHYSICS_INVERSE_TIMESTEP + 0.5) as u32;

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

impl Mul<u32> for Tick {
    type Output = Tick;

    fn mul(self, rhs: u32) -> Self::Output {
        Self {
            val: self.val * rhs,
        }
    }
}

#[derive(Resource)]
pub struct TickCounter {
    tick: Tick,
    since: Instant,
}

impl TickCounter {
    const DIAGNOSTIC_ITERS: u32 = 1000;

    fn new() -> Self {
        Self {
            tick: Tick { val: 0 },
            since: Instant::now(),
        }
    }

    pub fn diagnostic_iter(&self) -> bool {
        self.tick.val % Self::DIAGNOSTIC_ITERS == 0
    }

    pub fn should_save(&self) -> bool {
        self.tick.val % 100_000 == 0
    }
}

fn debug_tick_system(mut tick_counter: ResMut<TickCounter>, timesteps: Res<FixedTimesteps>) {
    tick_counter.tick.val += 1;

    if tick_counter.diagnostic_iter() {
        let tick = tick_counter.tick.val;

        let now = Instant::now();
        let dur = now.duration_since(tick_counter.since) / TickCounter::DIAGNOSTIC_ITERS;
        tick_counter.since = now;

        let info = timesteps.get_current().unwrap();
        let dur_iyes = info.timestep();

        info!(tick, ?dur, ?dur_iyes, "Tick");
    }
}

pub struct TickDebugPlugin;

impl Plugin for TickDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TickCounter::new())
            .add_engine_tick_system(debug_tick_system);
    }
}
