use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Vec2;
use engine::ability::seeker_rocket::SeekerRocket;
use engine::Energy;

use super::ObjectGraphics;
use crate::asset_handler::AssetHandler;
use crate::bar::Bar;
use crate::in_plane;

pub fn draw_seeker_rocket_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<SeekerRocket>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: assets.seeker_rocket.material.clone(),
                    mesh: assets.seeker_rocket.mesh.clone(),
                    ..Default::default()
                },
                Bar::<Energy>::new(0.1, Vec2::new(0.3, 0.08)),
                in_plane(),
                GlobalTransform::default(),
            ));
        });
    }
}
