use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use bevy::prelude::{Plugin, ResMut, Resource};
use tracing::info;

use crate::FixedTimestepSystem;

/// The timestep at which we run our game.
#[cfg(not(feature = "train"))]
pub const TIMESTEP: f32 = PHYSICS_TIMESTEP;
pub const PHYSICS_INVERSE_TIMESTEP: f32 = 60.0;
/// The timestep the physics engine sees.
pub const PHYSICS_TIMESTEP: f32 = 1.0 / PHYSICS_INVERSE_TIMESTEP;

/// Represents a duration in ticks rather than time.
#[derive(Default)]
pub struct Tick(pub u32);

impl Tick {
    /// Construct a new `Tick` from a duration using the engine `TIMESTEP`.
    pub const fn new(duration: Duration) -> Self {
        // This function is a bit funky as we're limited by what we can do in a
        // const function. E.g. No access to `round` or `max`.
        let ticks = (duration.as_secs_f32() * PHYSICS_INVERSE_TIMESTEP + 0.5) as u32;

        let val = if ticks == 0 { 1 } else { ticks };

        Self(val)
    }

    pub fn before_now(&self, counter: &TickCounter) -> bool {
        self.0 <= counter.tick
    }
}

impl Mul<u32> for Tick {
    type Output = Tick;

    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl From<u32> for Tick {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

#[derive(Resource)]
pub struct TickCounter {
    pub tick: u32,
    since: Instant,
}

impl TickCounter {
    const DIAGNOSTIC_ITERS: u32 = 10_000;

    fn new() -> Self {
        Self {
            tick: 1,
            since: Instant::now(),
        }
    }

    pub fn diagnostic_iter(&self) -> bool {
        self.tick % Self::DIAGNOSTIC_ITERS == 0
    }

    pub fn at(&self, tick: Tick) -> Tick {
        Tick(self.tick + tick.0)
    }

    #[cfg(not(feature = "train"))]
    pub fn should_save(&self) -> bool {
        false
    }

    #[cfg(feature = "train")]
    pub fn should_save(&self) -> bool {
        self.tick % 100_000 == 0
    }
}

fn tick_counter(mut tick_counter: ResMut<TickCounter>) {
    tick_counter.tick += 1;
}

fn debug_tick_system(mut tick_counter: ResMut<TickCounter>) {
    if tick_counter.diagnostic_iter() {
        let tick = tick_counter.tick;

        let now = Instant::now();
        let avg_dur = now.duration_since(tick_counter.since) / TickCounter::DIAGNOSTIC_ITERS;
        tick_counter.since = now;

        info!(tick, ?avg_dur, "Tick");
    }
}

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(TickCounter::new())
            .add_engine_tick_system(tick_counter)
            .add_engine_tick_system(debug_tick_system);
    }
}
