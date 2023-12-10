use std::{
    ops::{Add, Div, Sub},
    time::Instant,
};

use bevy_ecs::system::{ResMut, Resource};
use bevy_reflect::Reflect;
use bevy_utils::Duration;
use tracing::info;

/// The timestep at which we run our game.
pub const FREQUENCY: f32 = 60.0;
pub const TIMESTEP: f32 = 1.0 / FREQUENCY;

/// Represents an absolute time in frames since program start.
/// TODO: Ensure we're handling overflow.
#[derive(Default, Debug, Copy, Clone, Reflect, PartialEq, Eq)]
pub struct Frame(u32);

impl Frame {
    pub fn before_now(&self, counter: &FrameCounter) -> bool {
        self.0 <= counter.frame.0
    }
}

/// Represents a duration in ticks rather than time.
#[derive(Default, Debug, Copy, Clone, Reflect, PartialEq, Eq)]
pub struct Dur(u32);

impl Dur {
    pub fn new(ticks: u32) -> Self {
        Self(ticks)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn reduce_one(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

impl Add<Dur> for Frame {
    type Output = Frame;

    fn add(self, rhs: Dur) -> Self::Output {
        Frame(self.0.wrapping_add(rhs.0))
    }
}

impl Sub<Dur> for Frame {
    type Output = Frame;

    fn sub(self, rhs: Dur) -> Self::Output {
        Frame(self.0.wrapping_sub(rhs.0))
    }
}

impl Sub<Frame> for Frame {
    type Output = Dur;

    fn sub(self, rhs: Frame) -> Self::Output {
        Dur(self.0.wrapping_sub(rhs.0))
    }
}

impl Div<Dur> for Dur {
    type Output = f32;

    fn div(self, rhs: Dur) -> Self::Output {
        self.0 as f32 / rhs.0 as f32
    }
}

impl Div<Dur> for f32 {
    type Output = f32;

    fn div(self, rhs: Dur) -> Self::Output {
        self / rhs.0 as f32
    }
}

#[derive(Resource, Reflect)]
pub struct FrameCounter {
    pub frame: Frame,
    pub average_engine_frame: Duration,
    frame_begin: Instant,
    accumulated_time: Duration,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameCounter {
    pub const DIAGNOSTIC_ITERS: u32 = 1_000;

    pub fn new() -> Self {
        Self {
            frame: Frame::default(),
            frame_begin: Instant::now(),
            accumulated_time: Duration::ZERO,
            average_engine_frame: Duration::ZERO,
        }
    }

    pub fn diagnostic_iter(&self) -> bool {
        self.frame.0 % Self::DIAGNOSTIC_ITERS == 0
    }

    pub fn at(&self, dur: Dur) -> Frame {
        self.frame + dur
    }
}

/// Note: This system should be run before any others.
pub fn frame_counter(mut counter: ResMut<FrameCounter>) {
    counter.frame.0 += 1;
    counter.frame_begin = Instant::now();
}

/// Note: This system should be run after all others.
pub fn debug_frame_system(mut tick_counter: ResMut<FrameCounter>) {
    let frame_duration = tick_counter.frame_begin.elapsed();
    tick_counter.accumulated_time += frame_duration;

    if tick_counter.diagnostic_iter() {
        let tick = tick_counter.frame;

        let avg_dur = tick_counter.accumulated_time / FrameCounter::DIAGNOSTIC_ITERS;
        tick_counter.accumulated_time = Duration::ZERO;
        tick_counter.average_engine_frame = avg_dur;

        info!(tick = tick.0, ?avg_dur, "Tick");
    }
}
