use bevy::{
    app::{ScheduleRunnerPlugin, ScheduleRunnerSettings},
    prelude::App,
};
use gam::{time::TIMESTEP, GamPlugin, HeadlessDefaultPlugins};

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings {
            run_mode: bevy::app::RunMode::Loop {
                wait: Some(TIMESTEP),
            },
        })
        // .add_plugins(MinimalPlugins)
        .add_plugins(HeadlessDefaultPlugins)
        .add_plugin(ScheduleRunnerPlugin::default())
        // .add_plugins(ClientDefaultPlugins)
        // .add_plugins(DefaultPlugins.set(WindowPlugin {
        //     window: WindowDescriptor {
        //         present_mode: PresentMode::AutoNoVsync,
        //         ..default()
        //     },
        //     ..default()
        // }))
        .add_plugin(GamPlugin)
        // .add_plugin(GamClientPlugin)
        .add_plugin(gam::time::TickDebugPlugin)
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        // .add_plugin(bevy_rapier2d::render::RapierDebugRenderPlugin::default())
        .run();
}
