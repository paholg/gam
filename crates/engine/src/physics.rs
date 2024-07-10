use bevy_app::{App, Plugin};
use bevy_ecs::schedule::SystemConfigs;
use bevy_rapier3d::prelude::{
    NoUserData, PhysicsSet, RapierConfiguration, RapierPhysicsPlugin, TimestepMode,
};

use crate::{time::TIMESTEP, UP};

pub type RapierPlugin = RapierPhysicsPlugin<NoUserData>;

pub const G: f32 = 9.81;

pub struct PhysicsPlugin {
    config: RapierConfiguration,
    rapier: RapierPlugin,
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsPlugin {
    pub fn new() -> Self {
        let config = RapierConfiguration {
            gravity: UP * (-G),
            timestep_mode: TimestepMode::Fixed {
                dt: TIMESTEP,
                substeps: 1,
            },
            ..RapierConfiguration::new(1.0)
        };
        let rapier = RapierPlugin::default().with_default_system_setup(false);

        Self { config, rapier }
    }

    pub fn set1(&self) -> SystemConfigs {
        RapierPlugin::get_systems(PhysicsSet::SyncBackend)
    }

    pub fn set2(&self) -> SystemConfigs {
        RapierPlugin::get_systems(PhysicsSet::StepSimulation)
    }

    pub fn set3(&self) -> SystemConfigs {
        RapierPlugin::get_systems(PhysicsSet::Writeback)
    }
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config);
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
