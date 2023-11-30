use std::ops::MulAssign;

use bevy_ecs::{component::Component, system::Query};
use bevy_math::Vec2;
use bevy_rapier3d::prelude::{ExternalForce, ReadMassProperties, Velocity};
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;

use crate::{time::FREQUENCY, FootOffset, To2d, To3d};

/// The desired movement of an entity.
///
/// The magnitude of this vector represents the fraction of `MaxSpeed` that the
/// entity would like to move at. It is up to the setter to ensure it is always <= 1.0
#[derive(Component, Default)]
pub struct DesiredMove {
    pub dir: Vec2,
    pub can_fly: bool,
}

/// We currently move Characters by applying an impulse; this is the highest
/// impulse they can use.
#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct MaxSpeed {
    pub force: f32,
    pub speed: f32,
}

impl MulAssign<f32> for MaxSpeed {
    fn mul_assign(&mut self, rhs: f32) {
        self.force *= rhs;
        self.speed *= rhs;
    }
}

impl Default for MaxSpeed {
    fn default() -> Self {
        Self {
            force: 400.0,
            speed: 22.0,
        }
    }
}

pub fn apply_movement(
    mut query: Query<(
        &DesiredMove,
        &Transform,
        &mut ExternalForce,
        &Velocity,
        &MaxSpeed,
        &ReadMassProperties,
        &FootOffset,
    )>,
) {
    for (desired, transform, mut force, velocity, max_speed, mass, foot_offset) in &mut query {
        let y = transform.translation.y;
        // TODO: Determine if on ground betterer.
        let dir = if !desired.can_fly && (y + foot_offset.y < -0.1 || y + foot_offset.y > 0.1) {
            // TODO: Falling things, right now, can just try to slow down their
            // horizontal velocity. We'll want to replace this with something
            // better.
            -velocity.linvel.to_2d().normalize_or_zero()
        } else {
            desired.dir
        };
        let desired_v = max_speed.speed * dir;

        let delta_v = desired_v - velocity.linvel.to_2d();
        let delta_a = delta_v * FREQUENCY;
        let desired_f = delta_a * mass.mass;

        let delta_f = desired_f.clamp_length_max(max_speed.force).to_3d(0.0);

        force.force += delta_f;
    }
}
