use bevy::{
    prelude::{Color, Commands, Component, Entity, OnEnter, OnExit, Plugin, Query, With},
    ui::{BackgroundColor, Node, Val},
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
        Node {
            align_items: bevy::ui::AlignItems::Center,
            justify_content: bevy::ui::JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        BackgroundColor(Color::BLACK),
        SplashScreen,
    ));
}

fn splash_despawn(query: Query<Entity, With<SplashScreen>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
