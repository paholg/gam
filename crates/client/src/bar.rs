use std::fmt;
use std::marker::PhantomData;

use bevy::ecs::query::QueryData;
use bevy::pbr::NotShadowCaster;
use bevy::pbr::NotShadowReceiver;
use bevy::prelude::Added;
use bevy::prelude::BuildChildren;
use bevy::prelude::Bundle;
use bevy::prelude::Children;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::IntoSystemConfigs;
use bevy::prelude::Mesh;
use bevy::prelude::Parent;
use bevy::prelude::PbrBundle;
use bevy::prelude::Plugin;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Update;
use bevy::prelude::Vec2;
use bevy::prelude::Vec3;
use bevy::prelude::ViewVisibility;
use bevy::prelude::Visibility;
use bevy::prelude::With;
use bevy::prelude::Without;
use engine::Energy;
use engine::Health;
use tracing::warn;

use crate::asset_handler::AssetHandler;
use crate::in_plane;

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

pub trait HasBar: fmt::Debug {
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
        Self::new(0.36, Vec2::new(0.45, 0.16))
    }
}

impl Default for Bar<Energy> {
    fn default() -> Self {
        Self::new(0.44, Vec2::new(0.45, 0.16))
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

#[derive(QueryData)]
struct ParentQuery<T: Component> {
    entity: Entity,
    global_transform: &'static GlobalTransform,
    bar: &'static Bar<T>,
}

fn bar_add_system<T: Component + BarAssets + Default>(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    parents: Query<ParentQuery<T>, Added<Bar<T>>>,
) {
    for parent in parents.iter() {
        let (fg, bg, mesh) = T::assets(&assets);
        let recip_scale = parent.global_transform.compute_transform().scale.recip();
        let scale = recip_scale * parent.bar.size.extend(1.0);

        commands.entity(parent.entity).with_children(|builder| {
            builder
                .spawn(BarBundle::<T> {
                    transform: in_plane()
                        .with_translation(parent.bar.displacement * recip_scale)
                        .with_scale(scale),
                    ..Default::default()
                })
                .with_children(|builder| {
                    // Foreground
                    builder.spawn((
                        PbrBundle {
                            material: fg,
                            mesh: mesh.clone(),
                            ..Default::default()
                        },
                        BarChildMarker::<T>::default(),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                    // Background
                    builder.spawn((
                        PbrBundle {
                            material: bg,
                            mesh,
                            ..Default::default()
                        },
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                });
        });
    }
}

// We have a bit of a convoluted hierarchy here:
// An entity has the quantity we care about, T; entity_q.
// It has a child with graphics, including the Bar<T>; graphics_q.
// That has a child, with our transform; bar_q.
// That has children; the first is the foreground bar, the second is the
// background.
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
        let scale = (graphics_transform.scale * entity_transform.scale).recip();
        transform.rotation = rotation;
        transform.translation = rotation * bar.displacement * scale;

        // The foreground bar is the first child.
        let Some(&child) = children.iter().next() else {
            warn!("BarMarker does not have a child");
            return;
        };
        let Ok(mut bar_transform) = fgbar_q.get_mut(child) else {
            warn!("BarMarker's first child is incorrect!");
            return;
        };

        bar_transform.scale.x = percent;
        let offset = 0.5 * (1.0 - percent);
        bar_transform.translation.x = -offset;
    }
}
