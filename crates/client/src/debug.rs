use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::ChildBuild;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::Gizmos;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Text;
use bevy::prelude::Update;
use bevy::prelude::With;
use bevy::text::TextColor;
use bevy::text::TextFont;
use engine::ai::pathfind::HasPath;
use engine::debug::DebugText;
use rand::Rng;

use crate::ui::hud::Hud;
use crate::ui::hud::TEXT_COLOR;

fn rand_color() -> Color {
    let mut rng = rand::rng();

    let mut gen = || rng.random_range(0.0..=1.0);

    Color::srgb(gen(), gen(), gen())
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
            Text("".into()),
            TextFont::from_font_size(40.0),
            TextColor::from(TEXT_COLOR),
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
    text.0 = debug_text.get();
}
