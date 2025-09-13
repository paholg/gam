use std::ops::MulAssign;

use bevy::{
    ecs::{component::Component, system::Query},
    math::Vec3,
    reflect::Reflect,
    transform::components::Transform,
};
use bevy_rapier3d::prelude::Velocity;

use crate::{
    status_effect::TimeDilation,
    time::{FREQUENCY, TIMESTEP},
};

/// The desired movement of an entity.
///
/// The magnitude of this vector represents the fraction of `MaxSpeed` that the
/// entity would like to move at. It is up to the setter to ensure it is always
/// <= 1.0
#[derive(Component, Default, Debug)]
pub struct DesiredMove {
    pub vec: Vec3,
    pub can_fly: bool,
}

impl DesiredMove {
    pub fn reset(&mut self) {
        self.vec = Vec3::ZERO;
    }
}

/// We currently move Characters by applying an impulse; this is the highest
/// impulse they can use.
#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct MaxSpeed {
    pub accel: f32,
    pub speed: f32,
}

impl MulAssign<f32> for MaxSpeed {
    fn mul_assign(&mut self, rhs: f32) {
        self.accel *= rhs;
        self.speed *= rhs;
    }
}

impl Default for MaxSpeed {
    fn default() -> Self {
        Self {
            accel: 10.0,
            speed: 5.0,
        }
    }
}

impl MaxSpeed {
    pub const ZERO: MaxSpeed = MaxSpeed {
        accel: 0.0,
        speed: 0.0,
    };
}

pub fn apply_movement(
    mut query: Query<(
        &Transform,
        &DesiredMove,
        &mut Velocity,
        &MaxSpeed,
        &TimeDilation,
    )>,
) {
    for (transform, desired, mut velocity, max_speed, time_dilation) in &mut query {
        let factor = time_dilation.factor();
        let desired_v = transform.rotation * (max_speed.speed * desired.vec * factor);

        let desired_delta_v = desired_v - velocity.linvel;
        let delta_a = (desired_delta_v * FREQUENCY).clamp_length_max(max_speed.accel * factor);

        velocity.linvel += delta_a * TIMESTEP;
    }
}
