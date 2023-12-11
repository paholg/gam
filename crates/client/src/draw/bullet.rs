use bevy::prelude::{Added, Commands, Entity, Query, Res};
use engine::ability::bullet::Bullet;

use crate::asset_handler::AssetHandler;

use super::ObjectGraphics;

pub fn draw_bullet_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Bullet>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(ObjectGraphics {
            material: assets.bullet.material.clone(),
            mesh: assets.bullet.mesh.clone(),
            ..Default::default()
        });
    }
}
