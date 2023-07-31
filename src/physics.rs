use bevy::{
    prelude::{default, FixedUpdate, IntoSystemConfigs, IntoSystemSetConfigs, Plugin, Vec3},
    transform::TransformSystem,
};
use bevy_rapier3d::prelude::{
    NoUserData, PhysicsSet, RapierConfiguration, RapierPhysicsPlugin, TimestepMode,
};

use crate::time::PHYSICS_TIMESTEP;

pub type RapierPlugin = RapierPhysicsPlugin<NoUserData>;

pub struct PhysicsPlugin;

pub const G: f32 = 9.81;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let rapier_config = RapierConfiguration {
            gravity: Vec3::Z * (-G),
            timestep_mode: TimestepMode::Fixed {
                dt: PHYSICS_TIMESTEP,
                substeps: 1,
            },
            ..default()
        };

        // We need to manually add the systems so that they run at our
        // TIMESTEP tick rate instead of synced with framerate.
        // In `train` mode, we run as fast as possible, so we can use the
        // default system setup
        #[cfg(feature = "train")]
        app.insert_resource(rapier_config)
            .add_plugins(RapierPlugin::default());
        #[cfg(not(feature = "train"))]
        {
            app.insert_resource(rapier_config)
                .add_plugins(RapierPlugin::default().with_default_system_setup(false));
            app.configure_sets(
                FixedUpdate,
                (
                    PhysicsSet::SyncBackend,
                    PhysicsSet::SyncBackendFlush,
                    PhysicsSet::StepSimulation,
                    PhysicsSet::Writeback,
                )
                    .chain()
                    .before(TransformSystem::TransformPropagate),
            );

            app.add_systems(
                FixedUpdate,
                (
                    RapierPlugin::get_systems(PhysicsSet::SyncBackend)
                        .in_set(PhysicsSet::SyncBackend),
                    RapierPlugin::get_systems(PhysicsSet::SyncBackendFlush)
                        .in_set(PhysicsSet::SyncBackendFlush),
                    RapierPlugin::get_systems(PhysicsSet::StepSimulation)
                        .in_set(PhysicsSet::StepSimulation),
                    RapierPlugin::get_systems(PhysicsSet::Writeback).in_set(PhysicsSet::Writeback),
                ),
            );
        }
    }
}
