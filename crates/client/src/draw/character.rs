use bevy::app::Plugin;
use bevy::app::Update;
use bevy::core::FrameCount;
use bevy::ecs::system::SystemId;
use bevy::pbr::MeshMaterial3d;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::ChildBuild;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::In;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh3d;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Resource;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
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

use crate::asset_handler::AssetHandler;
use crate::bar::Bar;
use crate::in_plane;
use crate::Config;

pub struct CharacterPlugin;

#[derive(Resource)]
struct CharacterDeathCallbacks {
    player: SystemId<In<Entity>>,
    enemy: SystemId<In<Entity>>,
    ally: SystemId<In<Entity>>,
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
                    Mesh3d::from(assets.player.outline_mesh.clone()),
                    MeshMaterial3d::from(assets.player.outline_material.clone()),
                    in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn((
                    assets.player.scene.clone(),
                    Transform::from_translation(foot_offset.to_vec()),
                    Bar::<Health>::default(),
                    Bar::<Energy>::default(),
                ));
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
                    Mesh3d::from(assets.enemy.outline_mesh.clone()),
                    MeshMaterial3d::from(assets.enemy.outline_material.clone()),
                    in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn((
                    assets.enemy.scene.clone(),
                    Transform::from_translation(foot_offset.to_vec()),
                    Bar::<Health>::default(),
                    Bar::<Energy>::default(),
                ));
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
                    Mesh3d::from(assets.ally.outline_mesh.clone()),
                    MeshMaterial3d::from(assets.ally.outline_material.clone()),
                    in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn((
                    assets.ally.scene.clone(),
                    Transform::from_translation(foot_offset.to_vec()),
                    Bar::<Health>::default(),
                    Bar::<Energy>::default(),
                ));
            });
    }
}
