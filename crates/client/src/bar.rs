use std::{fmt, marker::PhantomData};

use bevy::{
    app::Startup,
    asset::Assets,
    color::palettes::css::{BLACK, GREEN},
    ecs::{
        hierarchy::ChildOf, query::QueryData, resource::Resource, schedule::IntoScheduleConfigs,
        system::ResMut,
    },
    light::{NotShadowCaster, NotShadowReceiver},
    math::primitives::Rectangle,
    pbr::MeshMaterial3d,
    prelude::{
        Added, Children, Color, Commands, Component, Entity, GlobalTransform, Handle, Mesh, Mesh3d,
        Plugin, Query, Res, StandardMaterial, Transform, Update, Vec2, Vec3, Visibility, With,
        Without,
    },
};
use engine::{Energy, Health};
use tracing::warn;

use crate::in_plane;

pub const BAR_OFFSET_Y: f32 = 0.01;

#[derive(Resource)]
struct BarAssets<T> {
    mesh: Handle<Mesh>,
    fg_material: Handle<StandardMaterial>,
    bg_material: Handle<StandardMaterial>,
    _marker: PhantomData<T>,
}

// NOTE: We use a very large value for depth_bias, because it doesn't play
// nicely with scale otherwise. If it's, say "1.0", and we have a small value
// for scale, it doesn't seem to help.
impl BarAssets<Health> {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let fg = StandardMaterial {
            base_color: GREEN.into(),
            unlit: true,
            depth_bias: 1000.0,
            ..Default::default()
        };
        let bg = StandardMaterial {
            base_color: BLACK.into(),
            unlit: true,
            depth_bias: -1000.0,
            ..Default::default()
        };
        BarAssets {
            mesh: meshes.add(Rectangle::new(1.0, 1.0)),
            fg_material: materials.add(fg),
            bg_material: materials.add(bg),
            _marker: PhantomData,
        }
    }
}
impl BarAssets<Energy> {
    fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        let fg = StandardMaterial {
            base_color: Color::linear_rgb(0.0, 0.2, 0.8),
            unlit: true,
            depth_bias: 1000.0,
            ..Default::default()
        };
        let bg = StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            depth_bias: -1000.0,
            ..Default::default()
        };
        BarAssets {
            mesh: meshes.add(Rectangle::new(1.0, 1.0)),
            fg_material: materials.add(fg),
            bg_material: materials.add(bg),
            _marker: PhantomData,
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
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

pub struct BarPlugin;

impl Plugin for BarPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, create_assets).add_systems(
            Update,
            (
                (bar_add_system::<Health>, bar_update_system::<Health>).chain(),
                (bar_add_system::<Energy>, bar_update_system::<Energy>).chain(),
            ),
        );
    }
}

fn create_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(BarAssets::<Health>::new(&mut meshes, &mut materials));
    commands.insert_resource(BarAssets::<Energy>::new(&mut meshes, &mut materials));
}

#[derive(QueryData)]
struct ParentQuery<T: Component> {
    entity: Entity,
    global_transform: &'static GlobalTransform,
    bar: &'static Bar<T>,
}

fn bar_add_system<T: Component + Default>(
    mut commands: Commands,
    assets: Res<BarAssets<T>>,
    parents: Query<ParentQuery<T>, Added<Bar<T>>>,
) {
    for parent in parents.iter() {
        let fg = assets.fg_material.clone();
        let bg = assets.bg_material.clone();
        let mesh = assets.mesh.clone();
        let recip_scale = parent.global_transform.compute_transform().scale.recip();
        let scale = recip_scale * parent.bar.size.extend(1.0);

        commands.entity(parent.entity).with_children(|builder| {
            builder
                .spawn((
                    BarMarker::<T>::default(),
                    in_plane()
                        .with_translation(parent.bar.displacement * recip_scale)
                        .with_scale(scale),
                ))
                .with_children(|builder| {
                    // Foreground
                    builder.spawn((
                        MeshMaterial3d::from(fg),
                        Mesh3d::from(mesh.clone()),
                        BarChildMarker::<T>::default(),
                        NotShadowCaster,
                        NotShadowReceiver,
                    ));
                    // Background
                    builder.spawn((
                        MeshMaterial3d::from(bg),
                        Mesh3d::from(mesh),
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
        (&ChildOf, &Transform, &Bar<T>),
        (Without<BarMarker<T>>, Without<BarChildMarker<T>>),
    >,
    mut bar_q: Query<
        (&ChildOf, &Children, &mut Transform),
        (With<BarMarker<T>>, Without<BarChildMarker<T>>),
    >,
    mut fgbar_q: Query<&mut Transform, (With<BarChildMarker<T>>, Without<BarMarker<T>>)>,
) {
    for (child_of, children, mut transform) in &mut bar_q {
        let Ok((grandchild_of, graphics_transform, bar)) = graphics_q.get(child_of.parent()) else {
            tracing::warn!(
                ?child_of,
                ?children,
                ?transform,
                "Could not get parent for bar"
            );
            continue;
        };

        let Ok((entity_transform, quantity)) = entity_q.get(grandchild_of.parent()) else {
            tracing::warn!(
                ?grandchild_of,
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
