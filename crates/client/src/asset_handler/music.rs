use bevy::{asset::LoadedFolder, prelude::Handle};

use super::Builder;

pub fn load_music(builder: &mut Builder) -> Handle<LoadedFolder> {
    builder
        .asset_server
        .load_folder("third-party/audio/Galacti-Chrons Weird Music Pack")
}
