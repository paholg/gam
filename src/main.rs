use bevy::{
    prelude::{App, Vec3},
    DefaultPlugins,
};
use bevy_rapier3d::prelude::{NoUserData, RapierConfiguration, RapierPhysicsPlugin};
use gam::{ability, healthbar::HealthbarPlugin, player_cooldown_system, system};

fn main() {
    let mut rapier_config = RapierConfiguration::default();
    rapier_config.gravity = Vec3::ZERO;
    App::new()
        .add_startup_system(gam::setup)
        .add_system(system::player_input)
        .add_system(system::update_cursor)
        .add_system(system::update_enemy_orientation)
        .add_system(system::die)
        .add_system(ability::hyper_sprint_system)
        .add_system(ability::shot_despawn_system)
        .add_system(ability::shot_hit_system)
        .add_system(ability::shot_miss_system)
        .add_system(player_cooldown_system)
        .add_plugins(DefaultPlugins)
        .add_plugin(HealthbarPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(rapier_config)
        // .add_plugin(bevy_rapier3d::render::RapierDebugRenderPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        .run();
}
