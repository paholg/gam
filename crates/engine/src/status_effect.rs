use bevy_ecs::bundle::Bundle;

pub use self::charge::Charge;
pub use self::phased::Phased;
pub use self::temperature::Temperature;
pub use self::time_dilation::TimeDilation;
use crate::time::Dur;

pub mod charge;
pub mod phased;
pub mod temperature;
pub mod time_dilation;

pub struct StatusProps {
    pub thermal_mass: f32,
    pub capacitance: f32,
}

impl From<StatusProps> for StatusBundle {
    fn from(value: StatusProps) -> Self {
        Self {
            temperatue: Temperature {
                temp: 0.0,
                thermal_mass: value.thermal_mass,
            },
            charge: Charge {
                potential: 0.0,
                capacitance: value.capacitance,
            },
            time_dilation: TimeDilation::default(),
            phase: Phased::default(),
        }
    }
}

/// Various status affects that might be on all entities in the world.
#[derive(Bundle, Debug)]
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
