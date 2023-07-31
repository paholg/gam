use bevy::{
    prelude::{
        Color, Commands, Component, DespawnRecursiveExt, Entity, NodeBundle, OnEnter, OnExit,
        Plugin, Query, With,
    },
    ui::{BackgroundColor, Style},
};

use crate::AppState;

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
