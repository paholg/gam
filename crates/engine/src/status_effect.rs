#![allow(unused)]
use bevy_ecs::{bundle::Bundle, component::Component};
use smallvec::SmallVec;

use crate::time::Dur;

/// Various status affects that might be on all entities in the world.
#[derive(Bundle, Debug, Default)]
pub struct StatusBundle {
    pub temperatue: Temperature,
    pub charge: Charge,
    pub time_dilation: TimeDilation,
    pub phase: Phased,
}

#[derive(Debug)]
pub struct Effect {
    duration: Dur,
    amount: f32,
}

impl Effect {
    pub fn new(duration: Dur, amount: f32) -> Self {
        Self { duration, amount }
    }
}

/// Temperature is measured as an abstract effect, rather than a number in
/// degress or Kelvin.
///
/// Room temperature is 0.0, colder is negative, warmer is positive.
///
/// The hotter something is, the harder it is to increase its temperature
/// (same for cold). Things should slowly return to 0.0 over time.
///
/// Probably mass also affects how hard it is to change something's temperature.
#[derive(Component, Debug, Default)]
pub struct Temperature {
    val: f32,
    effects: SmallVec<Effect, 8>,
}

/// Charge is measured as an abstract effect, rather than a unit in electrons or
/// Coulombs.
///
/// Neutral charge is 0.0, negative is negative, postive is postive.
///
/// Similarly charged things repel, differently charged things attract. When two
/// entities touch, they both take damage based on their charge difference (even
/// if one is neutral).
///
/// TODO: Should mass or volume or something affect damage taken by zaps?
#[derive(Component, Debug, Default)]
pub struct Charge {
    val: f32,
}

/// TimeDilation is measured in the abstract, where 0.0 is "normal" time,
/// negative is slower, positive is faster.
#[derive(Component, Debug, Default)]
pub struct TimeDilation {
    val: f32,
}

/// Phased is a boolean condition.
///
/// Phased things interact with only other phased things. So, for example, a
/// phased character can move through walls, is invulnerable to normal damage/
/// effects, but cannot hurt anyone. However, a phased enemy could fight them
/// like normal.
#[derive(Component, Debug, Default)]
pub struct Phased {
    val: bool,
}
