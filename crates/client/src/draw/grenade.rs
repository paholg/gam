use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::PbrBundle;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::Vec3;
use bevy::prelude::Without;
use bevy_rapier3d::prelude::Velocity;
use engine::ability::grenade::Grenade;
use engine::ability::grenade::GrenadeKind;
use engine::FootOffset;

use super::ObjectGraphics;
use crate::asset_handler::AssetHandler;
use crate::in_plane;

pub fn draw_grenade_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Grenade), Added<Grenade>>,
) {
    for (entity, grenade) in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        let (mesh, material) = match grenade.kind {
            GrenadeKind::Frag => (
                assets.frag_grenade.mesh.clone(),
                assets.frag_grenade.material.clone(),
            ),
            GrenadeKind::Heal => (
                assets.heal_grenade.mesh.clone(),
                assets.heal_grenade.material.clone(),
            ),
        };
        ecmds.insert(ObjectGraphics {
            material,
            mesh,
            ..Default::default()
        });
    }
}

#[derive(Component)]
pub struct HasOutline;

pub fn draw_grenade_outline_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Grenade, &Velocity, &FootOffset), Without<HasOutline>>,
) {
    for (entity, grenade, velocity, foot_offset) in &query {
        if velocity.linvel.length_squared() < 0.1 * 0.1 {
            let (mesh, material) = match grenade.kind {
                GrenadeKind::Frag => (
                    assets.frag_grenade.outline_mesh.clone(),
                    assets.frag_grenade.outline_material.clone(),
                ),
                GrenadeKind::Heal => (
                    assets.heal_grenade.outline_mesh.clone(),
                    assets.heal_grenade.outline_material.clone(),
                ),
            };
            let outline_entity = commands
                .spawn((
                    PbrBundle {
                        mesh,
                        material,
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
            commands
                .entity(entity)
                .insert(HasOutline)
                .push_children(&[outline_entity]);
        }
    }
}
