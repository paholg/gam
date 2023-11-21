use bevy::prelude::{
    Added, BuildChildren, Commands, Component, Entity, PbrBundle, Plugin, Query, Res, Transform,
    Update, With, Without,
};
use engine::{Player, Target};

use crate::asset_handler::AssetHandler;

/// A plugin for managing aiming, such as drawing and updating the cursor.
pub struct AimPlugin;

impl Plugin for AimPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (draw_target_system, update_target_system));
    }
}

#[derive(Component)]
struct CursorTarget {
    owner: Entity,
}

fn draw_target_system(
    mut commands: Commands,
    query: Query<(Entity, &Target), Added<Player>>,
    asset_handler: Res<AssetHandler>,
) {
    for (entity, target) in &query {
        let target_entity = commands
            .spawn((
                PbrBundle {
                    material: asset_handler.target.material.clone(),
                    mesh: asset_handler.target.mesh.clone(),
                    transform: Transform::from_translation(target.0.extend(0.0)),
                    ..Default::default()
                },
                CursorTarget { owner: entity },
            ))
            .id();
        commands.entity(entity).push_children(&[target_entity]);
    }
}

fn update_target_system(
    player_query: Query<(&Transform, &Target), With<Player>>,
    mut target_query: Query<(&mut Transform, &CursorTarget), Without<Player>>,
) {
    for (mut transform, target) in &mut target_query {
        if let Ok((player_transform, target)) = player_query.get(target.owner) {
            let rotation = player_transform.rotation.inverse();
            transform.rotation = rotation;
            transform.translation =
                rotation * (target.0.extend(0.0) - player_transform.translation);
        }
    }
}
