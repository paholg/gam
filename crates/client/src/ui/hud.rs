use std::time::{Duration, Instant};

use bevy::{
    prelude::{
        BuildChildren, Color, Commands, Component, NodeBundle, Plugin, Query, Res, ResMut,
        Resource, Startup, TextBundle, Update, With,
    },
    text::{Text, TextStyle},
    ui::{AlignItems, FlexDirection, JustifyContent, Style, Val},
};
use engine::{time::FrameCounter, NumAi};
use rust_i18n::t;

pub const TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(FpsTracker::new())
            .add_systems(Startup, persistent_ui_setup)
            .add_systems(
                Update,
                (score_update, frame_time_update, fps_update, fps_track),
            );
    }
}

#[derive(Component)]
pub struct Hud;

fn persistent_ui_setup(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(600.0),
                    height: Val::Px(20.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            Hud,
        ))
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
            parent.spawn((
                TextBundle::from_section(
                    render_frame_time(Duration::ZERO),
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        ..Default::default()
                    },
                ),
                FrameTime,
            ));
            parent.spawn((
                TextBundle::from_section(
                    render_frame_time(Duration::ZERO),
                    TextStyle {
                        font_size: 40.0,
                        color: TEXT_COLOR,
                        ..Default::default()
                    },
                ),
                FpsText,
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
    t!("menu.score", score = score).into_owned()
}

#[derive(Component)]
struct FrameTime;

fn frame_time_update(
    tick_counter: Res<FrameCounter>,
    mut query: Query<&mut Text, With<FrameTime>>,
) {
    let mut text = query.single_mut();
    text.sections[0].value = render_frame_time(tick_counter.average_engine_frame);
}

fn render_frame_time(time: Duration) -> String {
    let dur = format!("{time:?}");
    t!("menu.frame_time", time = dur).into_owned()
}

#[derive(Resource)]
struct FpsTracker {
    frame: u32,
    since: Instant,
    fps: f32,
}

impl FpsTracker {
    fn new() -> Self {
        Self {
            since: Instant::now(),
            frame: 0,
            fps: 0.0,
        }
    }
}

fn fps_track(mut fps: ResMut<FpsTracker>) {
    fps.frame += 1;

    if fps.frame % 100 == 0 {
        let dur = fps.since.elapsed();

        fps.fps = 100_f32 / dur.as_secs_f32();
        fps.since = Instant::now();
    }
}

#[derive(Component)]
struct FpsText;

fn fps_update(fps: Res<FpsTracker>, mut query: Query<&mut Text, With<FpsText>>) {
    let mut text = query.single_mut();
    text.sections[0].value = render_fps(fps.fps);
}

fn render_fps(fps: f32) -> String {
    let fps = format!("{fps:0.1}");
    t!("menu.fps", fps = fps).into_owned()
}
