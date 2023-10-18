use bevy::prelude::{in_state, IntoSystemConfigs, Plugin, Res, ResMut, Update};

use bevy_egui::{egui, EguiContexts, EguiPlugin};
use engine::{ability::Ability, AppState, NumAi};
use strum::IntoEnumIterator;

use crate::config::Config;

use super::BackgroundMusic;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // app.add_plugin(OverlayPlugin {
        //     font_size: 32.0,
        //     ..Default::default()
        // })
        // .add_system(text_update_system);
        app.add_plugins(EguiPlugin)
            .add_systems(Update, score_system)
            .add_systems(Update, config_menu.run_if(in_state(AppState::Paused)));
    }
}

fn score_system(mut contexts: EguiContexts, num_ai: Res<NumAi>, bg_music: Res<BackgroundMusic>) {
    egui::Window::new("Gam").show(contexts.ctx_mut(), |ui| {
        ui.heading(format!("Score: {}", num_ai.enemies));
        if let Some(name) = &bg_music.name {
            ui.heading(format!("Track: {}", name));
        }
    });
}

fn config_menu(mut contexts: EguiContexts, mut config: ResMut<Config>) {
    egui::Window::new("Config").show(contexts.ctx_mut(), |ui| {
        ui.heading("Abilities");
        for (i, ability) in config.player.abilities.iter_mut().enumerate() {
            ui.heading(format!("Ability {i}"));
            for alternative in Ability::iter() {
                ui.radio_value(ability, alternative, alternative.to_string());
            }
        }
    });
}
