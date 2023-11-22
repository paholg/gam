use bevy::{
    prelude::{
        in_state, BuildChildren, Color, Commands, Component, DespawnRecursiveExt, Entity,
        EventWriter, IntoSystemConfigs, NodeBundle, OnEnter, OnExit, Plugin, Query, Res, Startup,
        States, TextBundle, Update, With,
    },
    text::{Text, TextStyle},
    ui::{AlignItems, JustifyContent, Style, Val},
};

use bevy_ui_navigation::{events::Direction, prelude::NavRequest, NavigationPlugin};
use engine::{AppState, NumAi};
use leafwing_input_manager::prelude::ActionState;
use rust_i18n::t;

use crate::config::{MenuAction, UserAction};

/// The minimum amount of axis movement to register a menu selection change.
const MIN_MOVEMENT: f32 = 0.5;

const TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(NavigationPlugin::new())
            .add_state::<MenuState>()
            .add_systems(Startup, persistent_ui_setup)
            .add_systems(Update, (score_update,))
            .add_systems(Update, menu_nav_system.run_if(in_state(AppState::Paused)))
            .add_systems(OnEnter(AppState::Paused), setup_menu)
            .add_systems(OnExit(AppState::Paused), despawn::<Menu>);
    }
}

fn persistent_ui_setup(mut commands: Commands) {
    commands
        .spawn((NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(20.0),
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                ..Default::default()
            },
            ..Default::default()
        },))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    render_score(0),
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        ..Default::default()
                    },
                ),
                Score,
            ));
        });
}

#[derive(Component)]
struct Score;

fn score_update(num_ai: Res<NumAi>, mut query: Query<&mut Text, With<Score>>) {
    let mut text = query.single_mut();
    text.sections[0].value = render_score(num_ai.enemies);
}

fn render_score(score: usize) -> String {
    t!("menu.score", score = score)
}

fn menu_nav_system(
    player_query: Query<&ActionState<UserAction>>,
    mut events: EventWriter<NavRequest>,
) {
    for action_state in player_query.iter() {
        let movement = action_state
            .clamped_axis_pair(UserAction::Move)
            .unwrap_or_default();
        if movement.length_squared() > MIN_MOVEMENT * MIN_MOVEMENT {
            let x = movement.x();
            let y = movement.y();
            let direction = if x.abs() > y.abs() {
                if x < 0.0 {
                    Direction::West
                } else {
                    Direction::East
                }
            } else {
                #[allow(clippy::collapsible_else_if)] // symmetry
                if y < 0.0 {
                    Direction::South
                } else {
                    Direction::North
                }
            };
            events.send(NavRequest::Move(direction));
        }

        let menu_actions = action_state
            .get_just_pressed()
            .into_iter()
            .filter_map(|a| MenuAction::try_from(a).ok())
            .map(NavRequest::from);
        events.send_batch(menu_actions);
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
