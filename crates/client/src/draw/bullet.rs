use bevy::prelude::Added;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Res;
use engine::ability::bullet::Bullet;

use super::ObjectGraphics;
use crate::asset_handler::AssetHandler;

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
