use bevy::prelude::{App, PluginGroup};

fn main() {
    let mut app = App::new();

    app.add_plugins((
        bevy::DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }),
        engine::GamPlugin,
        client::GamClientPlugin,
    ));

    #[cfg(feature = "debug")]
    app.add_plugins((
        bevy::diagnostic::LogDiagnosticsPlugin::default(),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy_rapier3d::render::RapierDebugRenderPlugin::default(),
    ));

    app.run();
}
