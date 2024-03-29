use bevy::{
    core::FrameCount,
    prelude::{Commands, EventReader, Query, Res, ResMut, Transform},
};
use bevy_hanabi::EffectSpawner;
use bevy_kira_audio::{prelude::Volume, Audio, AudioControl};
use engine::{lifecycle::DeathEvent, Kind};

use crate::{asset_handler::AssetHandler, Config};

pub fn draw_death_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    mut event_reader: EventReader<DeathEvent>,
    frame: Res<FrameCount>,
) {
    for death in event_reader.read() {
        let effect = match death.kind {
            Kind::Other => None,
            Kind::Player => Some(&mut assets.player.despawn_effect),
            Kind::Enemy => Some(&mut assets.enemy.despawn_effect),
            Kind::Ally => Some(&mut assets.ally.despawn_effect),
            Kind::Bullet => Some(&mut assets.bullet.collision_effect),
            Kind::FragGrenade
            | Kind::HealGrenade
            | Kind::SeekerRocket
            | Kind::NeutrinoBall
            | Kind::TransportBeam => None,
        };

        if let Some(effect) = effect {
            effect.trigger(&mut commands, death.transform, &mut effects, &frame);
        }

        let sound = match death.kind {
            Kind::Other | Kind::NeutrinoBall | Kind::TransportBeam => None,
            Kind::Bullet => Some(assets.bullet.despawn_sound.clone()),
            Kind::Player
            | Kind::Enemy
            | Kind::Ally
            | Kind::FragGrenade
            | Kind::HealGrenade
            | Kind::SeekerRocket => Some(assets.player.despawn_sound.clone()),
        };

        if let Some(sound) = sound {
            audio
                .play(sound)
                .with_volume(Volume::Decibels(config.sound.effects_volume));
        }
    }
}
