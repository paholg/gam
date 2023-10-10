use bevy_app::{App, Plugin};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};

use crate::time::PHYSICS_TIMESTEP;

pub type RapierPlugin = RapierPhysicsPlugin<NoUserData>;

pub struct PhysicsPlugin;

pub const G: f32 = 9.81;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let rapier_config = RapierConfiguration {
            gravity: Vec3::Z * (-G),
            timestep_mode: TimestepMode::Fixed {
                dt: PHYSICS_TIMESTEP,
                substeps: 1,
            },
            ..Default::default()
        };

        app.insert_resource(rapier_config)
            .add_plugins(RapierPlugin::default().in_fixed_schedule());
    }
}
