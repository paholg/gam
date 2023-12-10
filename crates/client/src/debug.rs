use bevy::{
    prelude::{
        Added, BuildChildren, Color, Commands, Component, Entity, Gizmos, Plugin, Query, Res,
        TextBundle, Update, With,
    },
    text::{Text, TextStyle},
};
use engine::{ai::pathfind::HasPath, debug::DebugText};
use rand::{thread_rng, Rng};

use crate::ui::hud::{Hud, TEXT_COLOR};

fn rand_color() -> Color {
    let mut rng = thread_rng();

    let mut gen = || rng.gen_range(0.0..=1.0);

    Color::rgb(gen(), gen(), gen())
}

#[derive(Component)]
pub struct PathColor {
    color: Color,
}

pub fn draw_pathfinding_system(
    mut commands: Commands,
    query: Query<(Entity, &HasPath, Option<&PathColor>)>,
    mut gizmos: Gizmos,
) {
    for (entity, path, color) in &query {
        let color = match color {
            Some(color) => color.color,
            None => {
                let color = rand_color();
                commands.entity(entity).insert(PathColor { color });
                color
            }
        };

        let mut path = path.path.clone();
        for v in &mut path {
            v.y = 0.1;
        }

        gizmos.linestrip(path, color);
    }
}

pub struct DebugTextPlugin;

impl Plugin for DebugTextPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(DebugText::default())
            .add_systems(Update, (debug_text_setup, debug_text_update));
    }
}

#[derive(Component)]
struct DebugTextMarker;

fn debug_text_setup(mut commands: Commands, hud: Query<Entity, Added<Hud>>) {
    let Ok(hud_entity) = hud.get_single() else {
        return;
    };

    commands.entity(hud_entity).with_children(|builder| {
        builder.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 40.0,
                    color: TEXT_COLOR,
                    ..Default::default()
                },
            ),
            DebugTextMarker,
        ));
    });
}

fn debug_text_update(
    debug_text: Res<DebugText>,
    mut query: Query<&mut Text, With<DebugTextMarker>>,
) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };
    text.sections[0].value = debug_text.get();
}
