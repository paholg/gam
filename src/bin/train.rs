use bevy::{
    app::{ScheduleRunnerPlugin, ScheduleRunnerSettings},
    prelude::App,
};
use gam::{ai::a2c::A2CTrainerPlugin, GamPlugin, HeadlessDefaultPlugins};

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings {
            run_mode: bevy::app::RunMode::Loop { wait: None },
        })
        .add_plugins(HeadlessDefaultPlugins)
        .add_plugin(ScheduleRunnerPlugin::default())
        .add_plugin(GamPlugin)
        .add_plugin(A2CTrainerPlugin)
        .add_plugin(gam::time::TickDebugPlugin)
        // .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .run();
}