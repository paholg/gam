use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::{
        Added, BuildChildren, Commands, Entity, GlobalTransform, InheritedVisibility, PbrBundle,
        Query, Res, Transform, Vec3,
    },
};
use engine::{
    ability::gravity_ball::{GravityBall, GravityBallActivated},
    FootOffset,
};

use crate::{asset_handler::AssetHandler, in_plane};

use super::ObjectGraphics;

pub fn draw_neutrino_ball_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<GravityBall>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: assets.neutrino_ball.material.clone(),
                    mesh: assets.neutrino_ball.mesh.clone(),
                    ..Default::default()
                },
                Transform::IDENTITY,
                GlobalTransform::default(),
            ));
        });
    }
}

pub fn draw_neutrino_ball_outline_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), Added<GravityBallActivated>>,
) {
    for (entity, foot_offset) in &query {
        commands.entity(entity).with_children(|builder| {
            builder.spawn((
                PbrBundle {
                    mesh: assets.neutrino_ball.outline_mesh.clone(),
                    material: assets.neutrino_ball.outline_material.clone(),
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
        });
    }
}
