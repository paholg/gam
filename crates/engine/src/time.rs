use std::{
    ops::{Mul, SubAssign},
    time::Instant,
};

use bevy_ecs::system::{ResMut, Resource};
use bevy_reflect::Reflect;
use bevy_utils::Duration;
use tracing::info;

/// The timestep at which we run our game.
pub const FREQUENCY: f32 = 60.0;
pub const TIMESTEP: f32 = 1.0 / FREQUENCY;

/// Represents a duration in ticks rather than time.
#[derive(Default, Debug, Copy, Clone, Reflect, PartialEq, Eq)]
pub struct Tick(pub u32);

impl Tick {
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

impl SubAssign for Tick {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

#[derive(Resource, Reflect)]
pub struct TickCounter {
    pub tick: Tick,
    frame_begin: Instant,
    accumulated_time: Duration,
}

impl Default for TickCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl TickCounter {
    const DIAGNOSTIC_ITERS: u32 = 1_000;

    pub fn new() -> Self {
        Self {
            tick: Tick(0),
            frame_begin: Instant::now(),
            accumulated_time: Duration::ZERO,
        }
    }

    pub fn diagnostic_iter(&self) -> bool {
        self.tick.0 % Self::DIAGNOSTIC_ITERS == 0
    }

    pub fn at(&self, tick: Tick) -> Tick {
        Tick(self.tick.0 + tick.0)
    }
}

/// Note: This system should be run before any others.
pub fn tick_counter(mut tick_counter: ResMut<TickCounter>) {
    tick_counter.tick.0 += 1;
    tick_counter.frame_begin = Instant::now();
}

/// Note: This system should be run after all others.
pub fn debug_tick_system(mut tick_counter: ResMut<TickCounter>) {
    let frame_duration = tick_counter.frame_begin.elapsed();
    tick_counter.accumulated_time += frame_duration;

    if tick_counter.diagnostic_iter() {
        let tick = tick_counter.tick;

        let avg_dur = tick_counter.accumulated_time / TickCounter::DIAGNOSTIC_ITERS;
        tick_counter.accumulated_time = Duration::ZERO;

        info!(tick = tick.0, ?avg_dur, "Tick");
    }
}
