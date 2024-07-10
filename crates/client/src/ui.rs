use bevy::{
    prelude::{
        App, Commands, Component, DespawnRecursiveExt, Entity, OnEnter, OnExit, Plugin, Query,
        States, With,
    },
    state::app::AppExtStates,
};
// use bevy_ui_navigation::{events::Direction, prelude::NavRequest, NavigationPlugin};
use engine::AppState;
// use leafwing_input_manager::prelude::ActionState;

// use crate::config::{MenuAction, UserAction};

pub mod hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(hud::HudPlugin)
            .init_state::<MenuState>()
            // TODO: blocked on bevy-ui-navigation updating to Bevy 0.13
            // .add_plugins(NavigationPlugin::new())
            // .add_systems(Update, menu_nav_system.run_if(in_state(AppState::Paused)))
            .add_systems(OnEnter(AppState::Paused), setup_menu)
            .add_systems(OnExit(AppState::Paused), despawn::<Menu>);
    }
}

#[derive(Component)]
struct Menu;

#[derive(Default, Debug, Clone, Copy, Hash, States, PartialEq, Eq)]
enum MenuState {
    #[allow(unused)]
    Main,
    #[allow(unused)]
    Player,
    #[default]
    Off,
}

fn setup_menu(mut _commands: Commands) {}

fn despawn<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

// /// The minimum amount of axis movement to register a menu selection change.
// const MIN_MOVEMENT: f32 = 0.5;

// fn menu_nav_system(
//     player_query: Query<&ActionState<UserAction>>,
//     mut events: EventWriter<NavRequest>,
// ) {
//     for action_state in player_query.iter() {
//         let movement = action_state
//             .clamped_axis_pair(UserAction::Move)
//             .unwrap_or_default();
//         if movement.length_squared() > MIN_MOVEMENT * MIN_MOVEMENT {
//             let x = movement.x();
//             let y = movement.y();
//             let direction = if x.abs() > y.abs() {
//                 if x < 0.0 {
//                     Direction::West
//                 } else {
//                     Direction::East
//                 }
//             } else {
//                 #[allow(clippy::collapsible_else_if)] // symmetry
//                 if y < 0.0 {
//                     Direction::South
//                 } else {
//                     Direction::North
//                 }
//             };
//             events.send(NavRequest::Move(direction));
//         }

//         let menu_actions = action_state
//             .get_just_pressed()
//             .into_iter()
//             .filter_map(|a| MenuAction::try_from(a).ok())
//             .map(NavRequest::from);
//         events.send_batch(menu_actions);
//     }
// }
