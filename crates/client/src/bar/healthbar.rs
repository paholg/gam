use bevy::prelude::{
    default, Added, BuildChildren, Bundle, Children, Commands, Component, Entity, GlobalTransform,
    InheritedVisibility, Parent, PbrBundle, Query, Res, Transform, Vec2, Vec3, ViewVisibility,
    Visibility, With, Without,
};
use engine::Health;
use tracing::warn;

use crate::{asset_handler::AssetHandler, in_plane};

use super::BarMarker;

#[derive(Component)]
pub struct Healthbar {
    displacement: Vec3,
    pub size: Vec2,
}

impl Default for Healthbar {
    fn default() -> Self {
        Self {
            displacement: Vec3::new(0.0, 0.01, 0.7),
            size: Vec2::new(1.8, 0.3),
        }
    }
}

#[derive(Component, Default)]
pub struct HealthbarMarker;

#[derive(Bundle, Default)]
struct HealthbarBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    marker: HealthbarMarker,
}

pub fn add_healthbar_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    healthbars: Query<(Entity, &Healthbar), Added<Healthbar>>,
) {
    for (parent, healthbar) in healthbars.iter() {
        let bar = commands
            .spawn((
                PbrBundle {
                    material: assets.healthbar.fg_material.clone(),
                    mesh: assets.healthbar.mesh.clone(),
                    transform: in_plane().with_translation(healthbar.displacement),
                    ..default()
                },
                BarMarker,
            ))
            .id();
        let background = commands
            .spawn(PbrBundle {
                material: assets.healthbar.bg_material.clone(),
                mesh: assets.healthbar.mesh.clone(),
                transform: in_plane()
                    .with_translation(healthbar.displacement - Vec3::new(0.0, 0.01, 0.0)),
                ..default()
            })
            .id();
        let bundle = commands.spawn(HealthbarBundle::default()).id();
        commands.entity(bundle).push_children(&[bar, background]);
        commands.entity(parent).push_children(&[bundle]);
    }
}

pub fn healthbar_update_system(
    mut q_healthbar: Query<(&Parent, &Children, &mut Transform), With<HealthbarMarker>>,
    q_parent: Query<(&Transform, &Health, &Healthbar), Without<HealthbarMarker>>,
    mut q_child: Query<
        &mut Transform,
        (With<BarMarker>, Without<HealthbarMarker>, Without<Health>),
    >,
) {
    for (parent, children, mut transform) in q_healthbar.iter_mut() {
        let Ok((parent_transform, health, healthbar)) = q_parent.get(parent.get()) else {
            tracing::warn!(
                ?parent,
                ?children,
                ?transform,
                "Could not get parent for healthbar."
            );
            continue;
        };
        let healthiness = (health.cur / health.max).max(0.0);
        let rotation = parent_transform.rotation.inverse();
        transform.rotation = rotation;
        transform.translation = rotation * healthbar.displacement;

        // The bar is the first child, so we don't need to iterate over all of them.
        if let Some(&child) = children.iter().next() {
            if let Ok(mut bar_transform) = q_child.get_mut(child) {
                bar_transform.scale = Vec3::new(healthiness, 1.0, 1.0);
                let offset = healthbar.size.x * 0.5 * (1.0 - healthiness);
                bar_transform.translation = healthbar.displacement - Vec3::new(offset, 0.0, 0.0);
            } else {
                warn!("HealthbarMarker's first child is incorrect!");
            }
        } else {
            warn!("HealthbarMarker does not have a child");
        }
    }
}
