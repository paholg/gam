use bevy::prelude::{Component, Plugin, Update};

use self::{
    energybar::{add_energybar_system, energybar_update_system},
    healthbar::{add_healthbar_system, healthbar_update_system},
};

mod energybar;
mod healthbar;

pub use energybar::Energybar;
pub use healthbar::Healthbar;

#[derive(Component)]
pub struct BarMarker;

pub struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                healthbar_update_system,
                add_healthbar_system,
                energybar_update_system,
                add_energybar_system,
            ),
        );
    }
}
