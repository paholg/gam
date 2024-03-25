use bevy_ecs::{component::Component, system::Query};
use smallvec::SmallVec;

use super::Effect;
use crate::{time::Dur, Libm};

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
    pub const NONE: TimeDilation = TimeDilation {
        val: 1.0,
        effects: SmallVec::new(),
    };

    pub fn add_effect(&mut self, amount: f32, duration: Dur) {
        // We'll just add the effect, it will take place next frame.
        self.effects.push(Effect { duration, amount });
    }

    /// Return the speed-up or slow-down factor, where 1.0 is "normal" speed,
    /// 0.0 is stopped, 2.0 is double speed, etc.
    pub fn factor(&self) -> f32 {
        self.val
    }

    fn tick(&mut self) {
        // TODO: Should this be sum? product? something else?
        let effect: f32 = self.effects.iter().map(|e| e.amount).sum();

        // Things might get weird if time dilation affected how long time
        // dilation effects last. For now at least, they will always tick at the
        // "normal" rate.
        self.effects
            .retain(|e| !e.duration.tick(&TimeDilation::NONE));

        // Let's do exponential for < 0, linear for > 0 for now. Figure out
        // something that makes sense later.
        self.val = if effect.is_sign_negative() {
            Libm::exp(effect)
        } else {
            effect + 1.0
        };
    }
}

pub fn time_dilation_tick(mut query: Query<&mut TimeDilation>) {
    for mut dilation in &mut query {
        dilation.tick();
    }
}
