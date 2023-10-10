use bevy::prelude::{
    default, Added, BuildChildren, Bundle, Children, Commands, Component, ComputedVisibility,
    Entity, GlobalTransform, Parent, PbrBundle, Query, Res, Transform, Vec2, Vec3, Visibility,
    With, Without,
};
use engine::Energy;
use tracing::warn;

use crate::asset_handler::AssetHandler;

use super::BarMarker;

#[derive(Component)]
pub struct Energybar {
    displacement: Vec3,
    pub size: Vec2,
}

impl Default for Energybar {
    fn default() -> Self {
        Self {
            displacement: Vec3::new(0.0, -0.87, 0.01),
            size: Vec2::new(1.8, 0.3),
        }
    }
}

#[derive(Component)]
pub struct EnergybarMarker;

#[derive(Bundle)]
struct EnergybarBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    computed_visibility: ComputedVisibility,
    marker: EnergybarMarker,
}

pub fn add_energybar_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    energybars: Query<(Entity, &Energybar), Added<Energybar>>,
) {
    for (parent, energybar) in energybars.iter() {
        let bar = commands
            .spawn((
                PbrBundle {
                    material: assets.energybar.fg_material.clone(),
                    mesh: assets.energybar.mesh.clone(),
                    transform: Transform::from_translation(energybar.displacement),
                    ..default()
                },
                BarMarker,
            ))
            .id();
        let background = commands
            .spawn(PbrBundle {
                material: assets.energybar.bg_material.clone(),
                mesh: assets.energybar.mesh.clone(),
                transform: Transform::from_translation(
                    energybar.displacement - Vec3::new(0.0, 0.0, 0.01),
                ),
                ..default()
            })
            .id();
        let bundle = commands
            .spawn(EnergybarBundle {
                marker: EnergybarMarker,
                transform: Transform::default(),
                global_transform: GlobalTransform::default(),
                visibility: Visibility::Visible,
                computed_visibility: ComputedVisibility::default(),
            })
            .id();
        commands.entity(bundle).push_children(&[bar, background]);
        commands.entity(parent).push_children(&[bundle]);
    }
}

pub fn energybar_update_system(
    mut q_energybar: Query<(&Parent, &Children, &mut Transform), With<EnergybarMarker>>,
    q_parent: Query<(&Transform, &Energy, &Energybar), Without<EnergybarMarker>>,
    mut q_child: Query<
        &mut Transform,
        (With<BarMarker>, Without<EnergybarMarker>, Without<Energy>),
    >,
) {
    for (parent, children, mut transform) in q_energybar.iter_mut() {
        let (parent_transform, energy, energybar) = q_parent.get(parent.get()).unwrap();
        let energyiness = (energy.cur / energy.max).max(0.0);
        let rotation = parent_transform.rotation.inverse();
        transform.rotation = rotation;
        transform.translation = rotation * energybar.displacement;

        // The bar is the first child, so we don't need to iterate over all of them.
        if let Some(&child) = children.iter().next() {
            if let Ok(mut bar_transform) = q_child.get_mut(child) {
                bar_transform.scale = Vec3::new(energyiness, 1.0, 1.0);
                let offset = energybar.size.x * 0.5 * (1.0 - energyiness);
                bar_transform.translation = energybar.displacement - Vec3::new(offset, 0.0, 0.0);
            } else {
                warn!("EnergybarMarker's first child is incorrect!");
            }
        } else {
            warn!("EnergybarMarker does not have a child");
        }
    }
}
