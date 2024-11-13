use bevy::{
    math::bool,
    prelude::{App, Commands, PluginGroup, Res, Startup},
};
use clap::Parser;
use client::{debug::DebugTextPlugin, Config};
use engine::{player::PlayerInfo, Player};

#[derive(Parser)]
struct Args {
    /// If set, implies all other options
    #[arg(long)]
    all: bool,
    /// Whether to draw Ai paths
    #[arg(long)]
    paths: bool,
    /// Whether to draw the debug raycast cursor
    #[arg(long)]
    raycast_cursor: bool,
    /// Whether to show the egui inspector
    #[arg(long)]
    inspector: bool,
    /// Whether to show rapier colliders
    #[arg(long)]
    rapier: bool,
    /// Whether to log frame time
    #[arg(long)]
    frame_time: bool,
    /// Whether to show debug text
    #[arg(long)]
    debug_text: bool,
}

fn main() {
    let args = Args::parse();

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

    debug_stuff(&mut app, &args);

    app.run();
}

fn debug_stuff(app: &mut App, args: &Args) {
    app.add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default());

    if args.all || args.paths {
        app.add_systems(bevy::app::Update, client::debug::draw_pathfinding_system);
    }
    if args.all || args.raycast_cursor {
        app.insert_resource(
            bevy_mod_raycast::prelude::RaycastPluginState::<()>::default().with_debug_cursor(),
        );
    }
    if args.all || args.inspector {
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }
    if args.all || args.rapier {
        app.add_plugins(bevy_rapier3d::render::RapierDebugRenderPlugin::default());
    }
    if args.all || args.frame_time {
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    }
    if args.all || args.debug_text {
        app.add_plugins(DebugTextPlugin);
    }
}

fn player_spawner(mut commands: Commands, config: Res<Config>) {
    commands.spawn(PlayerInfo {
        abilities: config.player.abilities.clone(),
        // TODO: Set by server/engine for multiplayer
        handle: Player::new(0),
    });
}
