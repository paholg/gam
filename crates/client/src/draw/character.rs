use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::{
        Added, BuildChildren, Bundle, Commands, Entity, GlobalTransform, Handle,
        InheritedVisibility, PbrBundle, Query, Res, Scene, Transform, Vec3, ViewVisibility,
        Visibility, Without,
    },
};
use engine::{Ally, Enemy, Energy, FootOffset, Health, Player};

use super::raycast_scene::RaycastScene;
use crate::{asset_handler::AssetHandler, bar::Bar, in_plane};

#[derive(Bundle, Default)]
pub struct CharacterGraphics {
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
pub fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), Added<Player>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert(InheritedVisibility::default())
            .with_children(|builder| {
                builder.spawn((
                    PbrBundle {
                        mesh: assets.player.outline_mesh.clone(),
                        material: assets.player.outline_material.clone(),
                        transform: in_plane().with_translation(Vec3::new(
                            0.0,
                            foot_offset.y + 0.01,
                            0.0,
                        )),
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

pub fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), Added<Enemy>>,
) {
    for (entity, foot_offset) in query.iter() {
        let outline = commands
            .spawn((
                PbrBundle {
                    mesh: assets.enemy.outline_mesh.clone(),
                    material: assets.enemy.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        let graphics = commands
            .spawn(CharacterGraphics {
                scene: assets.enemy.scene.clone(),
                transform: Transform::from_translation(foot_offset.to_vec()),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(InheritedVisibility::default())
            .push_children(&[outline, graphics]);
    }
}

pub fn draw_ally_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), (Added<Ally>, Without<Player>)>,
) {
    for (entity, foot_offset) in query.iter() {
        let outline = commands
            .spawn((
                PbrBundle {
                    mesh: assets.ally.outline_mesh.clone(),
                    material: assets.ally.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        let graphics = commands
            .spawn(CharacterGraphics {
                scene: assets.ally.scene.clone(),
                transform: Transform::from_translation(foot_offset.to_vec()),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(InheritedVisibility::default())
            .push_children(&[outline, graphics]);
    }
}
