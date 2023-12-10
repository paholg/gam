use bevy_ecs::{component::Component, system::Query};

use super::TimeDilation;

const TEMP_LOSS_FACTOR: f32 = 0.1;

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

    // TODO: Introduce actual affects.
    fn tick(&mut self, time_dilation: &TimeDilation) {
        // Newton's law of coooling status that heat loss is directly
        // propotional between the difference in temperatures between an entity
        // and the environment. So let's try that.
        let delta = TEMP_LOSS_FACTOR * self.val * time_dilation.factor();
        self.val -= delta;
    }
}

pub fn temperature_tick(mut query: Query<(&mut Temperature, &TimeDilation)>) {
    for (mut temp, dilation) in &mut query {
        temp.tick(dilation);
    }
}
