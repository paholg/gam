use bevy::prelude::{
    Added, BuildChildren, Commands, Entity, GlobalTransform, InheritedVisibility, Query, Res, Vec2,
};
use engine::{ability::seeker_rocket::SeekerRocket, Energy};

use crate::{asset_handler::AssetHandler, bar::Bar, in_plane};

use super::ObjectGraphics;

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
