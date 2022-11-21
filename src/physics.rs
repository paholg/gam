use bevy::prelude::{default, CoreStage, Plugin, Vec2};
use bevy_rapier2d::prelude::{
    NoUserData, PhysicsStages, RapierConfiguration, RapierPhysicsPlugin, TimestepMode,
};
use iyes_loopless::prelude::AppLooplessFixedTimestepExt;

use crate::{
    time::{self, PHYSICS_TIMESTEP},
    CustomStage, FixedTimestepSystem, AFTER_CORESTAGE_UPDATE, BEFORE_CORESTAGE_LAST,
};

type RapierPlugin = RapierPhysicsPlugin<NoUserData>;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let rapier_config = RapierConfiguration {
            gravity: Vec2::ZERO,
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
            .add_plugin(RapierPlugin::default());
        #[cfg(not(feature = "train"))]
        app.insert_resource(rapier_config)
            .add_plugin(RapierPlugin::default().with_default_system_setup(false))
            // We add these stages just like RapierPhysicsPlugin::build would
            // have, but with fixed timesteps.
            // Analagous to PhysicsStages::SyncBackend:
            .add_fixed_timestep(time::TIMESTEP, AFTER_CORESTAGE_UPDATE)
            // Analagous to PhysicsStages::StepSimulation:
            .add_fixed_timestep_child_stage(AFTER_CORESTAGE_UPDATE)
            // Analagous to PhysicsStages::Writeback:
            .add_fixed_timestep_child_stage(AFTER_CORESTAGE_UPDATE)
            .add_fixed_timestep_before_stage(CoreStage::Last, time::TIMESTEP, BEFORE_CORESTAGE_LAST)
            .add_engine_tick_system_set_to_stage(
                CustomStage::PhysicsSyncBackend,
                RapierPlugin::get_systems(PhysicsStages::SyncBackend),
            )
            .add_engine_tick_system_set_to_stage(
                CustomStage::PhysicsStepSimulation,
                RapierPlugin::get_systems(PhysicsStages::StepSimulation),
            )
            .add_engine_tick_system_set_to_stage(
                CustomStage::PhysicsWriteback,
                RapierPlugin::get_systems(PhysicsStages::Writeback),
            )
            .add_engine_tick_system_set_to_stage(
                CustomStage::PhysicsDetectDespawn,
                RapierPlugin::get_systems(PhysicsStages::DetectDespawn),
            );
    }
}
