use bevy::{
    prelude::{default, App, PluginGroup},
    window::{PresentMode, WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};
use gam::{
    time::TickDebugPlugin, GamClientPlugin, GamPlugin,
};

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
        .add_plugin(TickDebugPlugin)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(bevy_rapier3d::render::RapierDebugRenderPlugin::default())
        .run();
}
