use bevy::{
    app::{App, Plugin},
    ecs::{schedule::ScheduleConfigs, system::ScheduleSystem},
};
use bevy_rapier3d::prelude::{NoUserData, PhysicsSet, RapierPhysicsPlugin, TimestepMode};

use crate::time::TIMESTEP;

pub type RapierPlugin = RapierPhysicsPlugin<NoUserData>;

pub struct PhysicsPlugin {
    timestep: TimestepMode,
    rapier: RapierPlugin,
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsPlugin {
    pub fn new() -> Self {
        let timestep = TimestepMode::Fixed {
            dt: TIMESTEP,
            substeps: 1,
        };
        let rapier = RapierPlugin::default()
            .with_default_system_setup(false)
            .with_custom_initialization(
            bevy_rapier3d::plugin::RapierContextInitialization::InitializeDefaultRapierContext {
                integration_parameters: bevy_rapier3d::rapier::prelude::IntegrationParameters {
                    dt: TIMESTEP,
                    ..Default::default()
                },
                rapier_configuration: bevy_rapier3d::plugin::RapierConfiguration::new(1.0),
            },
        );

        Self { rapier, timestep }
    }

    pub fn set1(&self) -> ScheduleConfigs<ScheduleSystem> {
        RapierPlugin::get_systems(PhysicsSet::SyncBackend)
    }

    pub fn set2(&self) -> ScheduleConfigs<ScheduleSystem> {
        RapierPlugin::get_systems(PhysicsSet::StepSimulation)
    }

    pub fn set3(&self) -> ScheduleConfigs<ScheduleSystem> {
        RapierPlugin::get_systems(PhysicsSet::Writeback)
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.timestep);
        self.rapier.build(app);
    }
}

#[cfg(test)]
mod test {
    use bevy_rapier3d::prelude::PhysicsSet;

    #[test]
    fn physics_sets() {
        let set = PhysicsSet::SyncBackend;
        // A simple test to make sure we get a compiler error if a new set is
        // added.
        match set {
            PhysicsSet::SyncBackend => (),
            PhysicsSet::StepSimulation => (),
            PhysicsSet::Writeback => (),
        }
    }
}
