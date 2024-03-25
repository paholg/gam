use std::{
    cmp::Ordering,
    ops::{Div, Sub},
    time::Instant,
};

use bevy_ecs::system::{ResMut, Resource};
use bevy_reflect::Reflect;
use bevy_utils::Duration;
use tracing::info;

use crate::status_effect::TimeDilation;

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
#[derive(Default, Debug, Copy, Clone, Reflect, PartialEq, PartialOrd)]
pub struct Dur(f32);

impl Dur {
    pub fn new(ticks: u32) -> Self {
        Self(ticks as f32)
    }

    pub fn is_done(self, time_dilation: &TimeDilation) -> bool {
        self.0.round() * time_dilation.factor() <= 0.0
    }

    pub fn is_positive(self) -> bool {
        self.0.is_sign_positive()
    }

    /// Tick down this duration, returning `true` if it has finished.
    pub fn tick(&mut self, time_dilation: &TimeDilation) -> bool {
        self.0 = (self.0 - time_dilation.factor()).max(0.0);
        self.is_done(time_dilation)
    }

    pub fn max(self, rhs: Dur) -> Dur {
        match self.partial_cmp(&rhs) {
            Some(Ordering::Greater) => self,
            Some(Ordering::Equal | Ordering::Less) => rhs,
            None => {
                debug_assert!(false, "Dur has invalid float value");
                if rhs.0.is_finite() {
                    rhs
                } else {
                    self
                }
            }
        }
    }
}

impl Div<Dur> for f32 {
    type Output = f32;

    fn div(self, rhs: Dur) -> Self::Output {
        self / rhs.0
    }
}

impl Div<Dur> for Dur {
    type Output = f32;

    fn div(self, rhs: Dur) -> Self::Output {
        self.0 / rhs.0
    }
}

impl Sub<Frame> for Frame {
    type Output = Dur;

    fn sub(self, rhs: Frame) -> Self::Output {
        Dur(self.0.wrapping_sub(rhs.0) as f32)
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
        let frame = self.frame.0 + dur.0.round() as u32;
        Frame(frame)
    }
}

/// Note: This system should be run before any others.
pub fn frame_counter(mut counter: ResMut<FrameCounter>) {
    counter.frame.0 += 1;
    counter.frame_begin = Instant::now();
}

/// Note: This system should be run after all others.
pub fn debug_frame_system(mut counter: ResMut<FrameCounter>) {
    let frame_duration = counter.frame_begin.elapsed();
    counter.accumulated_time += frame_duration;

    if counter.diagnostic_iter() {
        let tick = counter.frame;

        let avg_dur = counter.accumulated_time / FrameCounter::DIAGNOSTIC_ITERS;
        counter.accumulated_time = Duration::ZERO;
        counter.average_engine_frame = avg_dur;

        info!(tick = tick.0, ?avg_dur, "Tick");
    }
}
