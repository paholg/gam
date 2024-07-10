use bevy::{
    prelude::{
        Color, Commands, Component, DespawnRecursiveExt, Entity, NodeBundle, Plugin, Query, With,
    },
    state::state::{OnEnter, OnExit},
    ui::{BackgroundColor, Style, Val},
};
use engine::AppState;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(AppState::Loading), splash_setup);
        app.add_systems(OnExit(AppState::Loading), splash_despawn);
    }
}

#[derive(Component)]
struct SplashScreen;

fn splash_setup(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                align_items: bevy::ui::AlignItems::Center,
                justify_content: bevy::ui::JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        },
        SplashScreen,
    ));
}

fn splash_despawn(query: Query<Entity, With<SplashScreen>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
