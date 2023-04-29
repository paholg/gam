use bevy::prelude::{Plugin, Res};

use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::Score;

use super::BackgroundMusic;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_plugin(OverlayPlugin {
        //     font_size: 32.0,
        //     ..Default::default()
        // })
        // .add_system(text_update_system);
        app.add_plugin(EguiPlugin).add_system(score_system);
    }
}

fn score_system(mut contexts: EguiContexts, score: Res<Score>, bg_music: Res<BackgroundMusic>) {
    egui::Window::new("Gam").show(contexts.ctx_mut(), |ui| {
        ui.heading(format!("Score: {}", score.0));
        if let Some(name) = &bg_music.name {
            ui.heading(format!("Track: {}", name));
        }
    });
}
