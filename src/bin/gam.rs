use bevy::{
    prelude::{default, App, PluginGroup},
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};
use gam::{ai::a2c::A2CSamplerPlugin, GamClientPlugin, GamPlugin};

fn main() {
    App::new()
        // .add_plugins(HeadlessDefaultPlugins)
        // .add_plugins(ClientDefaultPlugins)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            },
            ..default()
        }))
        .add_plugin(GamPlugin)
        .add_plugin(GamClientPlugin)
        .add_plugin(gam::ai::a2c::A2CTrainerPlugin)
        // .add_plugin(A2CSamplerPlugin)
        .add_plugin(gam::time::TickDebugPlugin)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(bevy_rapier2d::render::RapierDebugRenderPlugin {
        //     enabled: true,
        //     always_on_top: true,
        //     ..default()
        // })
        .run();
}