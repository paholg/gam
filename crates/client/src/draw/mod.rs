use bevy::{
    app::Startup,
    prelude::{Plugin, Update},
};
use character::CharacterPlugin;
use explosion::ExplosionPlugin;

mod character;
mod death;
pub mod explosion;
pub mod level;
mod temperature;
mod time_dilation;

/// A plugin for spawning graphics for newly-created entities.
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Startup,
            (
                level::create_assets,
                time_dilation::setup_time_dilation,
                temperature::create_assets,
            ),
        )
        .add_systems(
            Update,
            (
                (
                    time_dilation::draw_time_dilation_system,
                    temperature::draw_temperature_system,
                    temperature::update_temperature_system,
                ),
                (level::draw_wall_system, level::draw_lights_system),
            ),
        )
        .add_plugins((CharacterPlugin, ExplosionPlugin));
    }
}
