use ability::AbilityPlugin;
use aim::AimPlugin;
use bar::BarPlugin;
use bevy::{
    ecs::{
        error::{BevyError, DefaultErrorHandler, ErrorContext},
        query::With,
        system::Single,
    },
    prelude::{Plugin, ResMut, Startup, Transform, Vec3},
    state::state::{OnEnter, OnExit},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_framepace::FramepaceSettings;
use config::ConfigPlugin;
use draw::DrawPlugin;
use engine::{AppState, UP};
use splash::SplashPlugin;

pub mod ability;
mod aim;
mod audio;
mod bar;
mod camera;
pub mod color_gradient;
mod config;
mod controls;
pub mod debug;
pub mod draw;
mod i18n;
mod particles;
mod shapes;
mod splash;
mod ui;
mod world;

pub use config::Config;
pub use controls::ControlPlugin;
use tracing::error;

use crate::{audio::MusicPlugin, camera::CameraPlugin};

/// Return a Transform such that things normally in the XY-plane will instead be
/// correctly oriented in the XZ plane.
pub fn in_plane() -> Transform {
    Transform::IDENTITY.looking_to(-UP, Vec3::Z)
}

/// This plugin includes user input and graphics.
pub struct GamClientPlugin;

impl Plugin for GamClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            AbilityPlugin,
            CameraPlugin,
            ConfigPlugin,
            GraphicsPlugin,
            MusicPlugin,
            SplashPlugin,
            bevy_framepace::FramepacePlugin,
            bevy_hanabi::HanabiPlugin,
        ))
        .insert_resource(DefaultErrorHandler(error_handler))
        .add_systems(Startup, setup_framepace)
        .add_systems(Startup, world::setup)
        .add_systems(OnEnter(AppState::Running), hide_cursor)
        .add_systems(OnExit(AppState::Running), show_cursor);
    }
}

struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((BarPlugin, ui::UiPlugin, DrawPlugin, AimPlugin));
    }
}

fn setup_framepace(mut settings: ResMut<FramepaceSettings>) {
    settings.limiter = bevy_framepace::Limiter::Auto;
}

pub fn error_handler(error: BevyError, ctx: ErrorContext) {
    error!(?error, ?ctx, "Bevy error");
}

// #[derive(Debug)]
// struct Hierarchy {
//     #[allow(dead_code)]
//     entity: Entity,
//     #[allow(dead_code)]
//     components: Vec<String>,
//     #[allow(dead_code)]
//     children: Vec<Hierarchy>,
// }

// /// Print the full hierarchy that includes this entity.
// pub fn print_hierarchy(
//     initial_entity: Entity,
//     world: &World,
//     q_parents: Query<&Children>,
//     q_children: Query<&ChildOf>,
// ) {
//     // First let's go to the top
//     let mut entity = initial_entity;
//     while let Ok(child_of) = q_children.get(entity) {
//         entity = child_of.parent();
//     }

//     let hierarchy = print_hierarchy_inner(entity, world, &q_parents);

//     println!("**************************************************");
//     println!("{:#?}", hierarchy);
//     println!("**************************************************");
// }

// fn print_hierarchy_inner(entity: Entity, world: &World, q_parents: &Query<&Children>) -> Hierarchy {
//     let components = world
//         .inspect_entity(entity)
//         .map(ComponentInfo::name)
//         .map(ToOwned::to_owned)
//         .collect();

//     let children = q_parents
//         .get(entity)
//         .map(|children| {
//             children
//                 .iter()
//                 .map(|child| print_hierarchy_inner(*child, world, q_parents))
//                 .collect()
//         })
//         .unwrap_or_default();

//     Hierarchy {
//         entity,
//         components,
//         children,
//     }
// }

fn hide_cursor(mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn show_cursor(mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor_options.visible = true;
    cursor_options.grab_mode = CursorGrabMode::None;
}
