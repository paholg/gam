use bevy::app::Startup;
use bevy::app::Update;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::ecs::system::Resource;
use bevy::prelude::App;
use bevy::prelude::Plugin;
use bevy::state::state::State;
use bevy_egui::egui;
use bevy_egui::egui::Slider;
use bevy_egui::egui::Ui;
use bevy_egui::EguiContexts;
use bevy_egui::EguiPlugin;
use engine::AppState;

use crate::config::AntiAliasing;
use crate::config::Audio;
use crate::config::Graphics;
use crate::config::MsaaSamples;
use crate::config::Sensitivity;
use crate::t;
use crate::Config;

pub mod hud;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((hud::HudPlugin, EguiPlugin))
            .insert_resource(Menu::default())
            .add_systems(Startup, setup)
            .add_systems(Update, menu);
    }
}

#[derive(Default, Resource)]
struct Menu {
    settings_tab: SettingsTab,
}

#[derive(PartialEq, Eq, Default)]
enum SettingsTab {
    #[default]
    Audio,
    Graphics,
}

fn setup(mut contexts: EguiContexts) {
    contexts.ctx_mut().all_styles_mut(|style| {
        style.override_text_style = Some(egui::TextStyle::Monospace);
    });
}

fn menu(
    mut contexts: EguiContexts,
    mut config: ResMut<Config>,
    mut menu: ResMut<Menu>,
    state: Res<State<AppState>>,
) {
    if state.get() != &AppState::Menu {
        return;
    }

    egui::SidePanel::left("settings").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut menu.settings_tab, SettingsTab::Audio, t!("audio"));
            ui.selectable_value(
                &mut menu.settings_tab,
                SettingsTab::Graphics,
                t!("graphics"),
            );
        });

        match menu.settings_tab {
            SettingsTab::Audio => audio(ui, &mut config.audio),
            SettingsTab::Graphics => graphics(ui, &mut config.graphics),
        }
        // ui.heading(t!("settings"));
        // ui.heading(t!("graphics"));
        // ui.checkbox(&mut config.graphics.bloom, t!("bloom"))
    });
}

fn audio(ui: &mut Ui, config: &mut Audio) {
    ui.add(Slider::new(&mut config.global_volume, -60.0..=0.0).text(t!("global")));
    ui.add(Slider::new(&mut config.effects_volume, -60.0..=0.0).text(t!("effects")));
    ui.add(Slider::new(&mut config.music_volume, -60.0..=0.0).text(t!("music")));
    ui.add(Slider::new(&mut config.speech_volume, -60.0..=0.0).text(t!("speech")));
}

fn graphics(ui: &mut Ui, config: &mut Graphics) {
    ui.checkbox(&mut config.bloom, t!("bloom"));

    ui.separator();
    ui.heading(t!("anti_aliasing"));

    ui.horizontal(|ui| {
        if ui
            .selectable_label(
                matches!(config.anti_aliasing, AntiAliasing::None),
                t!("none"),
            )
            .clicked()
        {
            config.anti_aliasing = AntiAliasing::None;
        }

        if ui
            .selectable_label(
                matches!(config.anti_aliasing, AntiAliasing::Fxaa { .. }),
                t!("fxaa"),
            )
            .clicked()
        {
            config.anti_aliasing = AntiAliasing::Fxaa {
                sensitivity: Default::default(),
            };
        }

        if ui
            .selectable_label(
                matches!(config.anti_aliasing, AntiAliasing::Msaa { .. }),
                t!("msaa"),
            )
            .clicked()
        {
            config.anti_aliasing = AntiAliasing::Msaa {
                samples: Default::default(),
            };
        }
    });

    ui.horizontal(|ui| match &mut config.anti_aliasing {
        AntiAliasing::None => (),
        AntiAliasing::Fxaa { sensitivity } => {
            ui.selectable_value(sensitivity, Sensitivity::Low, t!("sensitivity_low"));
            ui.selectable_value(sensitivity, Sensitivity::Medium, t!("sensitivity_medium"));
            ui.selectable_value(sensitivity, Sensitivity::High, t!("sensitivity_high"));
            ui.selectable_value(sensitivity, Sensitivity::Ultra, t!("sensitivity_ultra"));
            ui.selectable_value(sensitivity, Sensitivity::Extreme, t!("sensitivity_extreme"));
        }
        AntiAliasing::Msaa { samples } => {
            ui.selectable_value(samples, MsaaSamples::One, t!("samples_one"));
            ui.selectable_value(samples, MsaaSamples::Four, t!("samples_four"));
        }
    });
}
