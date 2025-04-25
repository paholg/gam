use bevy::app::Plugin;
use bevy::prelude::Component;
use gravity_ball::GravityBallPlugin;
use grenade::GrenadePlugin;
use gun::GunPlugin;
use rocket::RocketPlugin;
use transport::TransportBeamPlugin;

mod gravity_ball;
pub mod grenade;
mod gun;
pub mod rocket;
mod transport;

#[derive(Component)]
pub struct HasOutline;

pub struct AbilityPlugin;
impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            GravityBallPlugin,
            GrenadePlugin,
            GunPlugin,
            RocketPlugin,
            TransportBeamPlugin,
        ));
    }
}
