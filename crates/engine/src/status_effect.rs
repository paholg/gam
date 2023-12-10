use bevy_ecs::bundle::Bundle;

use crate::time::Dur;

pub use self::{
    charge::Charge, phased::Phased, temperature::Temperature, time_dilation::TimeDilation,
};

pub mod charge;
pub mod phased;
pub mod temperature;
pub mod time_dilation;

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
