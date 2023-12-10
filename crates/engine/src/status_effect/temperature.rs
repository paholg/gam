use bevy_ecs::{component::Component, system::Query, world::Mut};

use crate::{time::TIMESTEP, Health};

use super::TimeDilation;

const TEMP_LOSS_FACTOR: f32 = 1.0 * TIMESTEP;

const FIRE_DAMAGE_FACTOR: f32 = 1.0 * TIMESTEP;

/// Amount of heat energy an effect applies to an entity.
///
/// This is not in joules, but some abstract unit. Can be negative, to cool
/// things.
#[derive(Debug)]
pub struct Heat(f32);

impl Heat {
    pub fn new(amount: f32) -> Self {
        Self(amount)
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
}

impl Temperature {
    pub fn heat(&mut self, mass: f32, heat: Heat) {
        // TODO: Should different objects have different specific heats, or is
        // that too complicated?
        self.val += heat.0 / mass;
    }

    fn tick(&mut self, time_dilation: &TimeDilation, mut health: Mut<'_, Health>) {
        if self.val > 0.0 {
            let dmg = self.val * FIRE_DAMAGE_FACTOR;
            health.take(dmg, time_dilation);
        } else if self.val < 0.0 {
            // TODO: WHAT DO WHEN COLD?
        }

        if self.val.abs() < 0.1 {
            // Let's just zero-out near zero values.
            self.val = 0.0
        } else {
            // Newton's law of coooling status that heat loss is directly
            // propotional between the difference in temperatures between an entity
            // and the environment. So let's try that.
            let delta = TEMP_LOSS_FACTOR * self.val * time_dilation.factor();
            self.val -= delta;
        }
    }
}

pub fn temperature_tick(mut query: Query<(&mut Temperature, &TimeDilation, &mut Health)>) {
    for (mut temp, dilation, health) in &mut query {
        temp.tick(dilation, health);
    }
}
