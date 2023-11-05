use std::{
    ops::Mul,
    time::{Duration, Instant},
};

use bevy_app::{App, Plugin};
use bevy_ecs::system::{ResMut, Resource};
use bevy_reflect::Reflect;
use tracing::info;

use crate::EngineTickSystem;

/// The timestep at which we run our game.
pub const TIMESTEP: f32 = PHYSICS_TIMESTEP;
pub const PHYSICS_INVERSE_TIMESTEP: usize = 60;
/// The timestep the physics engine sees.
pub const PHYSICS_TIMESTEP: f32 = 1.0 / (PHYSICS_INVERSE_TIMESTEP as f32);

/// Represents a duration in ticks rather than time.
#[derive(Default, Debug, Copy, Clone, Reflect)]
pub struct Tick(pub u32);

impl Tick {
    /// Construct a new `Tick` from a duration using the engine `TIMESTEP`.
    pub const fn new(duration: Duration) -> Self {
        // This function is a bit funky as we're limited by what we can do in a
        // const function. E.g. No access to `round` or `max`.
        let ticks = (duration.as_secs_f32() * PHYSICS_INVERSE_TIMESTEP as f32 + 0.5) as u32;

        let val = if ticks == 0 { 1 } else { ticks };

        Self(val)
    }

    pub fn before_now(&self, counter: &TickCounter) -> bool {
        self.0 <= counter.tick.0
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

#[derive(Resource, Reflect)]
pub struct TickCounter {
    pub tick: Tick,
    since: Instant,
}

impl Default for TickCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl TickCounter {
    const DIAGNOSTIC_ITERS: u32 = 10_000;

    fn new() -> Self {
        Self {
            tick: Tick(0),
            since: Instant::now(),
        }
    }

    pub fn diagnostic_iter(&self) -> bool {
        self.tick.0 % Self::DIAGNOSTIC_ITERS == 0
    }

    pub fn at(&self, tick: Tick) -> Tick {
        Tick(self.tick.0 + tick.0)
    }
}

fn tick_counter(mut tick_counter: ResMut<TickCounter>) {
    tick_counter.tick.0 += 1;
}

fn debug_tick_system(mut tick_counter: ResMut<TickCounter>) {
    if tick_counter.diagnostic_iter() {
        let tick = tick_counter.tick;

        let now = Instant::now();
        let avg_dur = now.duration_since(tick_counter.since) / TickCounter::DIAGNOSTIC_ITERS;
        tick_counter.since = now;

        info!(tick = tick.0, ?avg_dur, "Tick");
    }
}

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TickCounter::new())
            .add_engine_tick_systems((tick_counter, debug_tick_system));
    }
}
