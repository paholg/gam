use bevy::core::FrameCount;
use bevy::prelude::Commands;
use bevy::prelude::EventReader;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Transform;
use bevy_hanabi::EffectInitializers;
use bevy_kira_audio::prelude::Volume;
use bevy_kira_audio::Audio;
use bevy_kira_audio::AudioControl;
use engine::lifecycle::DeathEvent;
use engine::Kind;

use crate::asset_handler::AssetHandler;
use crate::Config;

pub fn draw_death_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectInitializers)>,
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
