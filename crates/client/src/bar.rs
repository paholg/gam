use std::marker::PhantomData;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::{
        Added, BuildChildren, Bundle, Children, Commands, Component, Entity, GlobalTransform,
        Handle, InheritedVisibility, IntoSystemConfigs, Mesh, Parent, PbrBundle, Plugin, Query,
        Res, StandardMaterial, Transform, Update, Vec2, Vec3, ViewVisibility, Visibility, With,
        Without,
    },
};
use engine::{Energy, Health};
use tracing::warn;

use crate::{asset_handler::AssetHandler, in_plane};

pub const BAR_OFFSET_Y: f32 = 0.01;

#[derive(Component)]
pub struct BarMarker<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for BarMarker<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[derive(Component)]
pub struct BarChildMarker<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for BarChildMarker<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub trait HasBar {
    fn percent(&self) -> f32;
}

impl HasBar for Health {
    fn percent(&self) -> f32 {
        (self.cur / self.max).max(0.0)
    }
}

impl HasBar for Energy {
    fn percent(&self) -> f32 {
        (self.cur / self.max).max(0.0)
    }
}

#[derive(Component)]
pub struct Bar<T> {
    pub displacement: Vec3,
    pub size: Vec2,
    _marker: PhantomData<T>,
}

impl<T> Bar<T> {
    pub fn new(displacement: f32, size: Vec2) -> Self {
        Self {
            displacement: Vec3::new(0.0, BAR_OFFSET_Y, displacement),
            size,
            _marker: PhantomData,
        }
    }
}

impl Default for Bar<Health> {
    fn default() -> Self {
        Self::new(0.18, Vec2::new(0.45, 0.08))
    }
}

impl Default for Bar<Energy> {
    fn default() -> Self {
        Self::new(0.22, Vec2::new(0.45, 0.08))
    }
}

#[derive(Bundle, Default)]
struct BarBundle<T: Component> {
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    marker: BarMarker<T>,
}

pub struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                (bar_add_system::<Health>, bar_update_system::<Health>).chain(),
                (bar_add_system::<Energy>, bar_update_system::<Energy>).chain(),
            ),
        );
    }
}

trait BarAssets {
    fn assets(
        assets: &AssetHandler,
    ) -> (
        Handle<StandardMaterial>,
        Handle<StandardMaterial>,
        Handle<Mesh>,
    );
}

impl BarAssets for Health {
    fn assets(
        assets: &AssetHandler,
    ) -> (
        Handle<StandardMaterial>,
        Handle<StandardMaterial>,
        Handle<Mesh>,
    ) {
        (
            assets.healthbar.fg_material.clone(),
            assets.healthbar.bg_material.clone(),
            assets.healthbar.mesh.clone(),
        )
    }
}

impl BarAssets for Energy {
    fn assets(
        assets: &AssetHandler,
    ) -> (
        Handle<StandardMaterial>,
        Handle<StandardMaterial>,
        Handle<Mesh>,
    ) {
        (
            assets.energybar.fg_material.clone(),
            assets.energybar.bg_material.clone(),
            assets.energybar.mesh.clone(),
        )
    }
}

fn bar_add_system<T: Component + BarAssets + Default>(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    parents: Query<(Entity, &Bar<T>), Added<Bar<T>>>,
) {
    for (parent, bar) in parents.iter() {
        let (fg, bg, mesh) = T::assets(&assets);
        let foreground = commands
            .spawn((
                PbrBundle {
                    material: fg,
                    mesh: mesh.clone(),
                    transform: in_plane()
                        .with_translation(bar.displacement)
                        .with_scale(bar.size.extend(1.0)),
                    ..Default::default()
                },
                BarChildMarker::<T>::default(),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        let background = commands
            .spawn((
                PbrBundle {
                    material: bg,
                    mesh,
                    transform: in_plane()
                        .with_translation(bar.displacement - Vec3::new(0.0, BAR_OFFSET_Y, 0.0))
                        .with_scale(bar.size.extend(1.0)),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        let bundle = commands.spawn(BarBundle::<T>::default()).id();
        commands
            .entity(bundle)
            .push_children(&[foreground, background]);
        commands.entity(parent).push_children(&[bundle]);
    }
}

// We have a bit of a convoluted hierarchy here:
// An entity has the quantity we care about, T; entity_q.
// It has a child with graphics, including the Bar<T>; graphcs_q.
// That has a child, with our transform; bar_q.
// Finally, that has children, one of which we need to modify; fgbar_q.
pub fn bar_update_system<T: Component + HasBar>(
    entity_q: Query<(&Transform, &T), (Without<BarMarker<T>>, Without<BarChildMarker<T>>)>,
    graphics_q: Query<
        (&Parent, &Transform, &Bar<T>),
        (Without<BarMarker<T>>, Without<BarChildMarker<T>>),
    >,
    mut bar_q: Query<
        (&Parent, &Children, &mut Transform),
        (With<BarMarker<T>>, Without<BarChildMarker<T>>),
    >,
    mut fgbar_q: Query<&mut Transform, (With<BarChildMarker<T>>, Without<BarMarker<T>>)>,
) {
    for (parent, children, mut transform) in &mut bar_q {
        let Ok((grandparent, graphics_transform, bar)) = graphics_q.get(parent.get()) else {
            tracing::warn!(
                ?parent,
                ?children,
                ?transform,
                "Could not get parent for bar"
            );
            continue;
        };

        let Ok((entity_transform, quantity)) = entity_q.get(grandparent.get()) else {
            tracing::warn!(
                ?grandparent,
                ?children,
                ?transform,
                "Could not get grandparent for bar"
            );
            continue;
        };
        let percent = quantity.percent();
        let rotation = graphics_transform.rotation.inverse() * entity_transform.rotation.inverse();
        transform.rotation = rotation;
        transform.translation = rotation * bar.displacement;

        // The foreground bar is the first child, so we don't need to iterate over all of them.
        if let Some(&child) = children.iter().next() {
            if let Ok(mut bar_transform) = fgbar_q.get_mut(child) {
                bar_transform.scale.x = percent * bar.size.x;
                let offset = bar.size.x * 0.5 * (1.0 - percent);
                bar_transform.translation = bar.displacement - Vec3::new(offset, 0.0, 0.0);
            } else {
                warn!("BarMarker's first child is incorrect!");
            }
        } else {
            warn!("BarMarker does not have a child");
        }
    }
}
