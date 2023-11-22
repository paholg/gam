use bevy::prelude::{App, Commands, PluginGroup, Res, Startup};
use client::Config;
use engine::{player::PlayerInfo, Player};

fn main() {
    rust_i18n::set_locale("en");

    let mut app = App::new();

    app.add_plugins((
        bevy::DefaultPlugins.set(bevy::window::WindowPlugin {
            primary_window: Some(bevy::window::Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }),
        engine::GamPlugin,
        client::GamClientPlugin,
        client::ControlPlugin {
            player: Player::new(0),
        },
    ))
    .add_systems(Startup, player_spawner);

    #[cfg(feature = "debug")]
    app.add_plugins((
        bevy::diagnostic::LogDiagnosticsPlugin::default(),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy_rapier3d::render::RapierDebugRenderPlugin::default(),
    ))
    .insert_resource(
        bevy_mod_raycast::prelude::RaycastPluginState::<()>::default().with_debug_cursor(),
    );

    app.run();
}

fn player_spawner(mut commands: Commands, config: Res<Config>) {
    commands.spawn(PlayerInfo {
        abilities: config.player.abilities.clone(),
        // FIXME
        handle: Player::new(0),
    });
}
