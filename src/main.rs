use bevy::prelude::{App, PluginGroup};
use gam::GamPlugin;

fn main() {
    let mut app = App::new();

    #[cfg(feature = "graphics")]
    {
        app.add_plugins(bevy::DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(gam::GamClientPlugin);
    }

    // We load this plugin after the ClientPlugin, as that one loads any assets
    // we need and creates the AssetHandler resource.
    app.add_plugin(GamPlugin);

    #[cfg(not(feature = "graphics"))]
    {
        app.insert_resource(bevy::app::ScheduleRunnerSettings {
            run_mode: bevy::app::RunMode::Loop { wait: None },
        })
        .add_plugins(gam::HeadlessDefaultPlugins)
        .add_plugin(bevy::app::ScheduleRunnerPlugin::default());
    }
    #[cfg(feature = "debug")]
    {
        app.add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_plugin(bevy_rapier2d::render::RapierDebugRenderPlugin {
                enabled: true,
                always_on_top: true,
                ..Default::default()
            });
    }
    app.run();
}
