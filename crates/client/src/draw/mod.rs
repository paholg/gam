use bevy::prelude::Plugin;
use bevy::prelude::Update;
use character::CharacterPlugin;
use explosion::ExplosionPlugin;

mod character;
mod death;
pub mod explosion;
mod level;
mod temperature;
mod time_dilation;

/// A plugin for spawning graphics for newly-created entities.
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                // (
                //     bullet::draw_bullet_system,
                //     grenade::draw_grenade_system,
                //     grenade::draw_grenade_outline_system,
                //     rocket::draw_seeker_rocket_system,
                //     neutrino_ball::draw_neutrino_ball_system,
                //     neutrino_ball::draw_neutrino_ball_outline_system,
                //     transport::draw_transport_system,
                //     transport::update_transport_system,
                // ),
                (
                    time_dilation::draw_time_dilation_system,
                    temperature::draw_temperature_system,
                    temperature::update_temperature_system,
                ),
                (
                    level::draw_wall_system,
                    level::update_wall_system,
                    level::draw_lights_system,
                ),
            ),
        )
        .add_plugins((CharacterPlugin, ExplosionPlugin));
    }
}
