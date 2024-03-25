use bevy_ecs::{component::Component, query::QueryData, system::Query};

use super::TimeDilation;
use crate::{time::TIMESTEP, Health};

const CHARGE_LOSS_FACTOR: f32 = 0.1 * TIMESTEP;
const DISCHARGE_DAMAGE_FACTOR: f32 = 1.0;
const DISCHARGE_THRESHOLD: f32 = 0.1;
// TODO: Use or delete
// const CHARGE_FORCE_FACTOR: f32 = 1.0;

/// Charge represents electrostatic buildup, in electric potential. "Charge" is
/// perhaps a bad name.
///
/// Neutral charge is 0.0, negative is negative, postive is postive.
///
/// Similarly charged things repel, differently charged things attract. When two
/// entities touch, they both take damage based on their charge difference (even
/// if one is neutral).
///
/// When two charged objects come into combat, we use the formula for static
/// electricity discharge, E = 0.5*C*V*V, where E is energy, C is capacitance,
/// and V is electric potential, treating energy as proportional to damage done.
#[derive(Component, Debug)]
pub struct Charge {
    pub potential: f32,
    pub capacitance: f32,
}

impl Charge {
    fn tick(&mut self, time_dilation: &TimeDilation) {
        // TODO: How should charge decay? Let's just do it like temperature for
        // now.
        let delta = CHARGE_LOSS_FACTOR * self.potential * time_dilation.factor();
        self.potential -= delta;
    }

    pub fn should_discharge(&self, other: &Charge) -> bool {
        (self.potential - other.potential).abs() <= DISCHARGE_THRESHOLD
    }

    fn discharge(&mut self, other: &mut Charge) -> f32 {
        let new_potential = 0.5 * (self.potential + other.potential);
        let delta = (self.potential - new_potential).abs();
        self.potential = new_potential;
        other.potential = new_potential;

        delta * delta * DISCHARGE_DAMAGE_FACTOR * self.capacitance
    }
}

pub fn charge_tick(mut query: Query<(&mut Charge, &TimeDilation)>) {
    for (mut charge, dilation) in &mut query {
        charge.tick(dilation);
    }
}

// TODO: Do something with this.
#[derive(QueryData)]
#[query_data(mutable)]
struct CollisionQuery {
    charge: &'static mut Charge,
    health: Option<&'static mut Health>,
    time_dilation: Option<&'static TimeDilation>,
}

impl<'a> CollisionQueryItem<'a> {
    #[allow(unused)]
    fn discharge(&mut self, other: &mut CollisionQueryItem<'_>) {
        let damage = self.charge.discharge(&mut other.charge);
        self.take(damage);
        other.take(damage);
    }

    #[allow(unused)]
    fn take(&mut self, damage: f32) {
        if let (Some(h), Some(td)) = (self.health.as_mut(), self.time_dilation) {
            h.take(damage, td);
        }
    }
}
