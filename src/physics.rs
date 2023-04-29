use bevy::prelude::{default, Plugin, Vec3};
use bevy_rapier3d::prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin, TimestepMode};

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
        // #[cfg(feature = "train")]
        app.insert_resource(rapier_config)
            .add_plugin(RapierPlugin::default());
        // #[cfg(not(feature = "train"))]
        // {
        //     app.insert_resource(rapier_config)
        //         .add_plugin(RapierPlugin::default().with_default_system_setup(false));
        //     app.configure_sets(
        //         (
        //             PhysicsSet::SyncBackend,
        //             PhysicsSet::SyncBackendFlush,
        //             PhysicsSet::StepSimulation,
        //             PhysicsSet::Writeback,
        //         )
        //             .chain()
        //             .before(CoreSet::FixedUpdate),
        //     );

        //     app.add_systems(
        //         RapierPlugin::get_systems(PhysicsSet::SyncBackend)
        //             .in_base_set(PhysicsSet::SyncBackend),
        //     );
        //     app.add_systems(
        //         RapierPlugin::get_systems(PhysicsSet::SyncBackendFlush)
        //             .in_base_set(PhysicsSet::SyncBackendFlush),
        //     );
        //     app.add_systems(
        //         RapierPlugin::get_systems(PhysicsSet::StepSimulation)
        //             .in_base_set(PhysicsSet::StepSimulation),
        //     );
        //     app.add_systems(
        //         RapierPlugin::get_systems(PhysicsSet::Writeback).in_base_set(PhysicsSet::Writeback),
        //     );
        // }
    }
}
