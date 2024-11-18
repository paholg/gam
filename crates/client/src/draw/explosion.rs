use bevy::app::Plugin;
use bevy::app::Update;
use bevy::color::Color;
use bevy::prelude::Added;
use bevy::prelude::AlphaMode;
use bevy::prelude::Assets;
use bevy::prelude::BuildChildren;
use bevy::prelude::Commands;
use bevy::prelude::Component;
use bevy::prelude::Entity;
use bevy::prelude::GlobalTransform;
use bevy::prelude::Handle;
use bevy::prelude::InheritedVisibility;
use bevy::prelude::Mesh;
use bevy::prelude::Parent;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Sphere;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::With;
use bevy_rapier3d::prelude::Collider;
use engine::ability::explosion::Explosion;
use engine::ability::explosion::ExplosionKind;

use super::ObjectGraphics;
use crate::ability::rocket::RocketAssets;
use crate::color_gradient::ColorGradient;

#[derive(Component)]
pub struct ExplosionAssets {
    gradient: ColorGradient,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    // TODO: Add a sound.
}

impl ExplosionAssets {
    pub fn new(
        colors: ColorGradient,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Self {
        let initial_color = colors.get(0.0);
        ExplosionAssets {
            gradient: colors,
            mesh: meshes.add(Sphere::new(1.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgba(0.0, 0.0, 0.0, 0.5),
                emissive: initial_color,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            }),
        }
    }
}

pub struct ExplosionPlugin;
impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, (draw_explosion, update_explosion));
    }
}

#[derive(Component)]
struct ExplosionGraphics;

fn draw_explosion(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Explosion, &Collider), Added<Explosion>>,
    rocket_assets: Res<RocketAssets>,
) {
    for (entity, explosion, collider) in &query {
        let explosion_assets = match explosion.kind {
            ExplosionKind::FragGrenade => todo!(),
            ExplosionKind::HealGrenade => todo!(),
            ExplosionKind::SeekerRocket => &rocket_assets.explosion,
        };
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

fn update_explosion(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Parent, &Handle<StandardMaterial>), With<ExplosionGraphics>>,
    parent_q: Query<(&Explosion, &Transform)>,
    rocket_assets: Res<RocketAssets>,
) {
    for (parent, material) in &mut query {
        let Ok((explosion, parent_transform)) = parent_q.get(parent.get()) else {
            tracing::warn!("ExplosionGraphics missing parent");
            continue;
        };
        // Assume scale is same in all dimensions.
        let radius = parent_transform.scale.x;

        let min_radius = explosion.min_radius;
        let max_radius = explosion.max_radius;

        let frac = (radius - min_radius) / (max_radius - min_radius);
        let explosion_assets = match explosion.kind {
            ExplosionKind::FragGrenade => todo!(),
            ExplosionKind::HealGrenade => todo!(),
            ExplosionKind::SeekerRocket => &rocket_assets.explosion,
        };

        let color = explosion_assets.gradient.get(frac);
        materials.get_mut(material).unwrap().emissive = color;
    }
}
