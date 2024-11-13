use bevy::prelude::Added;
use bevy::prelude::Assets;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Parent;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::With;
use bevy_rapier3d::prelude::Collider;
use engine::death_callback::Explosion;
use engine::death_callback::ExplosionKind;

use super::ObjectGraphics;
use crate::asset_handler::explosion::ExplosionAssets;
use crate::asset_handler::AssetHandler;

#[derive(Component)]
pub struct ExplosionGraphics;

fn explosion_assets(assets: &AssetHandler, kind: ExplosionKind) -> &ExplosionAssets {
    match kind {
        ExplosionKind::FragGrenade => &assets.frag_grenade.explosion,
        ExplosionKind::HealGrenade => &assets.heal_grenade.explosion,
        ExplosionKind::SeekerRocket => &assets.seeker_rocket.explosion,
    }
}

pub fn draw_explosion_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Explosion, &Collider), Added<Explosion>>,
) {
    for (entity, explosion, collider) in &query {
        let explosion_assets = explosion_assets(&assets, explosion.kind);
        let radius = collider.as_ball().unwrap().radius();

        // Clone the material because we're going to mutate it. Probably we
        // could do this better.
        let material = materials.get(&explosion_assets.material).unwrap().clone();

        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        commands.entity(entity).with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: materials.add(material),
                    mesh: explosion_assets.mesh.clone(),
                    ..Default::default()
                },
                Transform::from_scale(Vec3::splat(radius)),
                GlobalTransform::default(),
                ExplosionGraphics,
            ));
        });
    }
}

pub fn update_explosion_system(
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Parent, &mut Transform, &Handle<StandardMaterial>), With<ExplosionGraphics>>,
    parent_q: Query<(&Explosion, &Collider)>,
) {
    for (parent, mut transform, material) in &mut query {
        let Ok((explosion, collider)) = parent_q.get(parent.get()) else {
            tracing::warn!("ExplosionGraphics missing parent");
            continue;
        };
        let radius = collider.as_ball().unwrap().radius();

        let min_radius = explosion.min_radius;
        let max_radius = explosion.max_radius;

        let frac = (radius - min_radius) / (max_radius - min_radius);
        let explosion_assets = explosion_assets(&assets, explosion.kind);

        let color = explosion_assets.gradient.get(frac);
        materials.get_mut(material).unwrap().emissive = color;

        transform.scale = Vec3::splat(radius);
    }
}
