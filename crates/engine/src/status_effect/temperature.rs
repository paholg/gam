use bevy_ecs::{component::Component, system::Query, world::Mut};

use super::TimeDilation;
use crate::{time::TIMESTEP, Health};

const TEMP_LOSS_FACTOR: f32 = 1.0 * TIMESTEP;

const FIRE_DAMAGE_FACTOR: f32 = 1.0 * TIMESTEP;

/// Temperature is measured as an abstract effect, rather than a number in
/// degress or Kelvin.
///
/// Room temperature is 0.0, colder is negative, warmer is positive.
///
/// The hotter something is, the harder it is to increase its temperature
/// (same for cold). Things should slowly return to 0.0 over time.
///
/// Probably mass also affects how hard it is to change something's temperature.
#[derive(Component, Debug)]
pub struct Temperature {
    pub temp: f32,
    /// A thermal_mass of 1.0 means that 1.0 unit of heat causes 1.0 unit of
    /// temperature gain. Higher thermal mass means less temperature gain/loss.
    pub thermal_mass: f32,
}

impl Temperature {
    pub fn heat(&mut self, heat: f32) {
        self.temp += heat / self.thermal_mass;
    }

    fn tick(&mut self, time_dilation: &TimeDilation, mut health: Mut<'_, Health>) {
        if self.temp > 0.0 {
            let dmg = self.temp * FIRE_DAMAGE_FACTOR;
            health.take(dmg, time_dilation);
        } else if self.temp < 0.0 {
            // TODO: WHAT DO WHEN COLD?
            // Cold lowers your acceleration, increases your max speed, and
            // gives vulnerability to blunt & piercing damage.
        }

        if self.temp.abs() < 0.1 {
            // Let's just zero-out near zero values.
            self.temp = 0.0
        } else {
            // Newton's law of coooling status that heat loss is directly
            // propotional between the difference in temperatures between an entity
            // and the environment. So let's try that.
            let delta = TEMP_LOSS_FACTOR * self.temp * time_dilation.factor();
            self.temp -= delta;
        }
    }
}

pub fn temperature_tick(mut query: Query<(&mut Temperature, &TimeDilation, &mut Health)>) {
    for (mut temp, dilation, health) in &mut query {
        temp.tick(dilation, health);
    }
}
