use bevy::prelude::{Plugin, Res};
use bevy_debug_text_overlay::{screen_print, OverlayPlugin};

use crate::NumAi;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(OverlayPlugin {
            font_size: 32.0,
            ..Default::default()
        })
        .add_system(text_update_system);
    }
}

fn text_update_system(num_ai: Res<NumAi>) {
    screen_print!("Score: {}", num_ai.enemies);
}
