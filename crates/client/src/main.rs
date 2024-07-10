use bevy::{
    log::LogPlugin,
    math::bool,
    prelude::{App, Commands, PluginGroup, Res, Startup},
    utils::HashMap,
};
use clap::Parser;
use client::{debug::DebugTextPlugin, Config};
use engine::{player::PlayerInfo, Player};
use itertools::Itertools;
use tracing::Level;
use tracing_subscriber::EnvFilter;

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
    /// Whether to log at debug level
    #[arg(long)]
    debug_logs: bool,
}

fn main() {
    rust_i18n::set_locale("en");

    let args = Args::parse();

    let mut app = App::new();
    args.setup_logging();

    app.add_plugins((
        bevy::DefaultPlugins
            .set(bevy::window::WindowPlugin {
                primary_window: Some(bevy::window::Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .build()
            .disable::<LogPlugin>(),
        engine::GamPlugin,
        client::GamClientPlugin,
        client::ControlPlugin {
            player: Player::new(0),
        },
    ))
    .add_systems(Startup, player_spawner);

    args.debug_stuff(&mut app);

    app.run();
}

impl Args {
    fn check(&self, field: bool) -> bool {
        self.all || field
    }

    fn log_level(&self) -> Level {
        if self.check(self.debug_logs) {
            Level::DEBUG
        } else {
            Level::INFO
        }
    }

    fn debug_stuff(&self, app: &mut App) {
        app.add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default());

        if self.check(self.paths) {
            // TODO: pathfind
            //     app.add_systems(bevy::app::Update, client::debug::draw_pathfinding_system);
        }
        if self.check(self.raycast_cursor) {
            app.insert_resource(
                bevy_mod_raycast::prelude::RaycastPluginState::<()>::default().with_debug_cursor(),
            );
        }
        if self.check(self.inspector) {
            app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        }
        if self.check(self.rapier) {
            app.add_plugins(bevy_rapier3d::render::RapierDebugRenderPlugin::default());
        }
        if self.check(self.frame_time) {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
        }
        if self.check(self.debug_text) {
            app.add_plugins(DebugTextPlugin);
        }
    }

    fn setup_logging(&self) {
        let level = self.log_level();

        let filter_string = LogFilter::new()
            .set("engine", level)
            .set("client", level)
            // By default, every audio file loaded results in multiple logs :(
            .set("symphonia_core", Level::WARN)
            .set("symphonia_format_ogg", Level::WARN)
            .build();

        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| {
                EnvFilter::builder()
                    .with_default_directive(Level::INFO.into())
                    .parse(filter_string)
            })
            .unwrap();
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            // .with_file(true)
            // .with_line_number(true)
            .init();
    }
}

#[derive(Default)]
struct LogFilter {
    inner: HashMap<&'static str, Level>,
}

impl LogFilter {
    fn new() -> Self {
        Self::default()
    }

    fn set(mut self, key: &'static str, level: Level) -> Self {
        self.inner.insert(key, level);
        self
    }

    fn build(&self) -> String {
        [Level::INFO.to_string()]
            .into_iter()
            .chain(
                self.inner
                    .iter()
                    .map(|(key, level)| format!("{key}={level}")),
            )
            .join(",")
    }
}

fn player_spawner(mut commands: Commands, config: Res<Config>) {
    commands.spawn(PlayerInfo {
        ability_ids: config.player.ability_ids.clone(),
        // TODO: Set by server/engine for multiplayer
        handle: Player::new(0),
    });
}
