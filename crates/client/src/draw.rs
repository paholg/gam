use bevy::prelude::Bundle;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh;
use bevy::prelude::Plugin;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Update;
use bevy::prelude::ViewVisibility;
use bevy::prelude::Visibility;
use character::CharacterPlugin;

mod character;
mod death;
// mod explosion;
// mod grenade;
mod level;
// mod neutrino_ball;
mod raycast_scene;
// mod rocket;
mod temperature;
mod time_dilation;
// mod transport;

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
                    raycast_scene::raycast_scene_system,
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
        .add_plugins(CharacterPlugin);
    }
}

#[derive(Bundle, Default)]
pub struct ObjectGraphics {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}
