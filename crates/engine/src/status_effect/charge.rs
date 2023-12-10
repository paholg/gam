#![allow(unused)] // FIXME
use bevy_ecs::{component::Component, query::WorldQuery, system::Query};

use crate::{time::TIMESTEP, Health};

use super::TimeDilation;

const CHARGE_LOSS_FACTOR: f32 = 0.1 * TIMESTEP;
const DISCHARGE_DAMAGE_FACTOR: f32 = 1.0;
const DISCHARGE_THRESHOLD: f32 = 0.5;
const CHARGE_FORCE_FACTOR: f32 = 1.0;

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
    pub val: f32,
}

impl Charge {
    fn tick(&mut self, time_dilation: &TimeDilation) {
        // TODO: How should charge decay? Let's just do it like temperature for
        // now.
        let delta = CHARGE_LOSS_FACTOR * self.val * time_dilation.factor();
        self.val -= delta;
    }

    pub fn should_discharge(&self, other: &Charge) -> bool {
        (self.val - other.val).abs() <= DISCHARGE_THRESHOLD
    }

    fn discharge(&mut self, other: &mut Charge) -> f32 {
        let new_charge = 0.5 * (self.val + other.val);
        let delta = (self.val - new_charge).abs();
        self.val = new_charge;
        other.val = new_charge;

        delta * DISCHARGE_DAMAGE_FACTOR
    }
}

pub fn charge_tick(mut query: Query<(&mut Charge, &TimeDilation)>) {
    for (mut charge, dilation) in &mut query {
        charge.tick(dilation);
    }
}

#[derive(WorldQuery)]
#[world_query(mutable)]
struct CollisionQuery {
    charge: &'static mut Charge,
    health: Option<&'static mut Health>,
    time_dilation: Option<&'static TimeDilation>,
}

impl<'a> CollisionQueryItem<'a> {
    fn discharge(&mut self, other: &mut CollisionQueryItem<'_>) {
        let damage = self.charge.discharge(&mut other.charge);
        self.take(damage);
        other.take(damage);
    }

    fn take(&mut self, damage: f32) {
        if let (Some(h), Some(td)) = (self.health.as_mut(), self.time_dilation) {
            h.take(damage, td);
        }
    }
}
