use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::PbrBundle;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use engine::ability::neutrino_ball::NeutrinoBall;
use engine::ability::neutrino_ball::NeutrinoBallActivated;
use engine::FootOffset;

use super::ObjectGraphics;
use crate::asset_handler::AssetHandler;
use crate::in_plane;

pub fn draw_neutrino_ball_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<NeutrinoBall>>,
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
    query: Query<(Entity, &FootOffset), Added<NeutrinoBallActivated>>,
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
