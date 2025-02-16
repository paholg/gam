use bevy_app::App;
use bevy_app::Plugin;
use bevy_ecs::schedule::SystemConfigs;
use bevy_rapier3d::prelude::NoUserData;
use bevy_rapier3d::prelude::PhysicsSet;
use bevy_rapier3d::prelude::RapierPhysicsPlugin;
use bevy_rapier3d::prelude::TimestepMode;

use crate::time::TIMESTEP;

pub type RapierPlugin = RapierPhysicsPlugin<NoUserData>;

pub const G: f32 = 9.81;

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
        let rapier = RapierPlugin::default().with_default_system_setup(false);

        Self { rapier, timestep }
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
