use bevy::app::Plugin;
use bevy::app::Update;
use bevy::core::FrameCount;
use bevy::ecs::system::SystemId;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Bundle;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::In;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::PbrBundle;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::Scene;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::ViewVisibility;
use bevy::prelude::Visibility;
use bevy::prelude::Without;
use bevy_hanabi::EffectInitializers;
use bevy_kira_audio::prelude::Volume;
use bevy_kira_audio::Audio;
use bevy_kira_audio::AudioControl;
use engine::lifecycle::ClientDeathCallback;
use engine::Ally;
use engine::Enemy;
use engine::Energy;
use engine::FootOffset;
use engine::Health;
use engine::Player;

use super::raycast_scene::RaycastScene;
use crate::asset_handler::AssetHandler;
use crate::bar::Bar;
use crate::in_plane;
use crate::Config;

pub struct CharacterPlugin;

#[derive(Resource)]
struct CharacterDeathCallbacks {
    player: SystemId<Entity>,
    enemy: SystemId<Entity>,
    ally: SystemId<Entity>,
}

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let callbacks = CharacterDeathCallbacks {
            player: app.register_system(player_death_system),
            enemy: app.register_system(enemy_death_system),
            ally: app.register_system(ally_death_system),
        };

        app.insert_resource(callbacks).add_systems(
            Update,
            (draw_player_system, draw_enemy_system, draw_ally_system),
        );
    }
}

fn player_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectInitializers>>,
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectInitializers)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.player.despawn_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.player.despawn_sound.clone();
    audio
        .play(sound)
        .with_volume(Volume::Decibels(config.sound.effects_volume));
}

fn enemy_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectInitializers>>,
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectInitializers)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.enemy.despawn_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.enemy.despawn_sound.clone();
    audio
        .play(sound)
        .with_volume(Volume::Decibels(config.sound.effects_volume));
}

fn ally_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectInitializers>>,
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectInitializers)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.ally.despawn_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.ally.despawn_sound.clone();
    audio
        .play(sound)
        .with_volume(Volume::Decibels(config.sound.effects_volume));
}

#[derive(Bundle, Default)]
struct CharacterGraphics {
    healthbar: Bar<Health>,
    energybar: Bar<Energy>,
    scene: Handle<Scene>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    raycast: RaycastScene,
    transform: Transform,
    global_transform: GlobalTransform,
}

fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    callbacks: Res<CharacterDeathCallbacks>,
    query: Query<(Entity, &FootOffset), Added<Player>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert((
                InheritedVisibility::default(),
                ClientDeathCallback::new(callbacks.player),
            ))
            .with_children(|builder| {
                builder.spawn((
                    PbrBundle {
                        mesh: assets.player.outline_mesh.clone(),
                        material: assets.player.outline_material.clone(),
                        transform: in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                        ..Default::default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn(CharacterGraphics {
                    scene: assets.player.scene.clone(),
                    transform: Transform::from_translation(foot_offset.to_vec()),
                    ..Default::default()
                });
            });
    }
}

fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    callbacks: Res<CharacterDeathCallbacks>,
    query: Query<(Entity, &FootOffset), Added<Enemy>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert((
                InheritedVisibility::default(),
                ClientDeathCallback::new(callbacks.enemy),
            ))
            .with_children(|builder| {
                builder.spawn((
                    PbrBundle {
                        mesh: assets.enemy.outline_mesh.clone(),
                        material: assets.enemy.outline_material.clone(),
                        transform: in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                        ..Default::default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn(CharacterGraphics {
                    scene: assets.enemy.scene.clone(),
                    transform: Transform::from_translation(foot_offset.to_vec()),
                    ..Default::default()
                });
            });
    }
}

fn draw_ally_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    callbacks: Res<CharacterDeathCallbacks>,
    query: Query<(Entity, &FootOffset), (Added<Ally>, Without<Player>)>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert((
                InheritedVisibility::default(),
                ClientDeathCallback::new(callbacks.ally),
            ))
            .with_children(|builder| {
                builder.spawn((
                    PbrBundle {
                        mesh: assets.ally.outline_mesh.clone(),
                        material: assets.ally.outline_material.clone(),
                        transform: in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                        ..Default::default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn(CharacterGraphics {
                    scene: assets.ally.scene.clone(),
                    transform: Transform::from_translation(foot_offset.to_vec()),
                    ..Default::default()
                });
            });
    }
}
