#![allow(unused)]
use std::{fmt, ops::Mul};

use bevy_ecs::{bundle::Bundle, component::Component, system::Query};
use bevy_rapier3d::prelude::ReadMassProperties;
use smallvec::SmallVec;

use crate::{time::Dur, Libm};

const TEMP_LOSS_FACTOR: f32 = 0.1;
const CHARGE_LOSS_FACTOR: f32 = 0.1;

/// Various status affects that might be on all entities in the world.
#[derive(Bundle, Debug, Default)]
pub struct StatusBundle {
    pub temperatue: Temperature,
    pub charge: Charge,
    pub time_dilation: TimeDilation,
    pub phase: Phased,
}

#[derive(Debug)]
struct Effect {
    amount: f32,
    duration: Dur,
}

impl Effect {
    fn tick(&mut self) {
        self.duration.reduce_one();
    }
}

pub trait StatusEffect: Component + fmt::Debug {
    fn tick(&mut self);
}

pub fn status_effect_system<S: StatusEffect>(mut query: Query<&mut S>) {
    for mut effect in &mut query {
        effect.tick();
    }
}

/// Amount of heat energy an effect applies to an entity.
///
/// This is not in joules, but some abstract unit. Can be negative, to cool
/// things.
#[derive(Debug)]
pub struct Heat(f32);

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
}

impl Temperature {
    pub fn heat(&mut self, mass: f32, heat: Heat) {
        // TODO: Should different objects have different specific heats, or is
        // that too complicated?
        self.val += heat.0 / mass;
    }
}

impl StatusEffect for Temperature {
    // TODO: Introduce actual affects.
    fn tick(&mut self) {
        // Newton's law of coooling status that heat loss is directly
        // propotional between the difference in temperatures between an entity
        // and the environment. So let's try that.
        let delta = TEMP_LOSS_FACTOR * self.val;
        self.val -= delta;
    }
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

impl StatusEffect for Charge {
    fn tick(&mut self) {
        // TODO: How should charge decay? Let's just do it like temperature for
        // now.
        let delta = CHARGE_LOSS_FACTOR * self.val;
        self.val -= delta;
    }
}

/// TimeDilation represents a factor that we multiply "time" values by, such as
/// speed, energy regen, and duration. It is also a factor in how much damage
/// you take.
///
/// Multiple time dilation effects can be in place at the same time; we take the
/// sum and perform some math to achieve a factor that can be multiplied by
/// time-things.
// TODO: We currently only account for time dilation for move speed and damage.
#[derive(Component, Debug)]
pub struct TimeDilation {
    val: f32,
    effects: SmallVec<Effect, 2>,
}

impl Default for TimeDilation {
    fn default() -> Self {
        Self {
            val: 1.0,
            effects: Default::default(),
        }
    }
}

impl TimeDilation {
    pub fn add_effect(&mut self, amount: f32, duration: Dur) {
        // We'll just add the effect, it will take place next frame.
        self.effects.push(Effect { duration, amount });
    }

    /// Return the speed-up or slow-down factor, where 1.0 is "normal" speed,
    /// 0.0 is stopped, 2.0 is double speed, etc.
    pub fn factor(&self) -> f32 {
        self.val
    }
}

impl StatusEffect for TimeDilation {
    fn tick(&mut self) {
        // TODO: Should this be sum? product? something else?
        let effect: f32 = self.effects.iter().map(|e| e.amount).sum();

        self.effects.iter_mut().for_each(Effect::tick);
        self.effects.retain(|e| e.duration.is_positive());

        // Let's do exponential for < 0, linear for > 0 for now. Figure out
        // something that makes sense later.
        self.val = if effect.is_sign_negative() {
            Libm::exp(effect)
        } else {
            effect + 1.0
        };
    }
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
    duration: Dur,
}

impl Phased {
    fn toggle(&mut self) {
        self.val = !self.val
    }
}

impl StatusEffect for Phased {
    fn tick(&mut self) {
        if self.duration.is_positive() {
            self.duration.reduce_one();
            if self.duration.is_zero() {
                self.toggle();
            }
        }
    }
}
