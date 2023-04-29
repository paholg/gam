use bevy::{
    prelude::{
        Color, Commands, Component, DespawnRecursiveExt, Entity,
        IntoSystemAppConfig, NodeBundle, OnEnter, OnExit, Plugin, Query, With,
    },
    ui::{BackgroundColor, Size, Style, Val},
};

use crate::AppState;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(splash_setup.in_schedule(OnEnter(AppState::Loading)))
            .add_system(splash_despawn.in_schedule(OnExit(AppState::Loading)));
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
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
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
