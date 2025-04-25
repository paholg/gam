use bevy::prelude::App;
use bevy::prelude::AppExtStates;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::DespawnRecursiveExt;
use bevy::prelude::Entity;
use bevy::prelude::OnEnter;
use bevy::prelude::OnExit;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::States;
use bevy::prelude::With;
use engine::AppState;

pub mod hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(hud::HudPlugin)
            .init_state::<MenuState>()
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
