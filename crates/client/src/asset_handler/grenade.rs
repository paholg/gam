use bevy::color::LinearRgba;
use bevy::math::primitives::Sphere;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;

use super::explosion::ExplosionAssets;
use super::Builder;
use crate::color_gradient::ColorGradient;
use crate::shapes::HollowPolygon;

pub struct GrenadeAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
}

impl GrenadeAssets {
    pub fn frag(builder: &mut Builder) -> Self {
        let frag_grenade_material = StandardMaterial {
            emissive: LinearRgba::rgb(10.0, 0.0, 1.0),
            ..Default::default()
        };

        GrenadeAssets {
            mesh: builder
                .meshes
                .add(Sphere::new(builder.props.frag_grenade.radius)),
            material: builder.materials.add(frag_grenade_material.clone()),
            outline_mesh: builder.meshes.add(HollowPolygon {
                radius: builder.props.frag_grenade.explosion.max_radius,
                thickness: 0.06,
                vertices: 60,
            }),
            outline_material: builder.materials.add(frag_grenade_material),
            explosion: ExplosionAssets::new(
                builder,
                ColorGradient::new([
                    (0.0, LinearRgba::new(50.0, 12.0, 0.0, 0.2)),
                    (0.5, LinearRgba::new(100.0, 25.0, 0.0, 0.2)),
                    (0.8, LinearRgba::new(2.0, 2.0, 2.0, 0.2)),
                    (1.0, LinearRgba::new(0.0, 0.0, 0.0, 0.1)),
                ]),
            ),
        }
    }

    pub fn heal(builder: &mut Builder) -> Self {
        let heal_grenade_material = StandardMaterial {
            emissive: LinearRgba::rgb(0.0, 10.0, 1.0),
            ..Default::default()
        };

        GrenadeAssets {
            mesh: builder
                .meshes
                .add(Sphere::new(builder.props.heal_grenade.radius)),
            outline_mesh: builder.meshes.add(HollowPolygon {
                radius: builder.props.heal_grenade.explosion.max_radius,
                thickness: 0.06,
                vertices: 60,
            }),
            material: builder.materials.add(heal_grenade_material.clone()),
            outline_material: builder.materials.add(heal_grenade_material),
            explosion: ExplosionAssets::new(
                builder,
                ColorGradient::new([
                    (0.0, LinearRgba::new(0.0, 50.0, 0.0, 0.2)),
                    (0.5, LinearRgba::new(0.0, 100.0, 0.0, 0.2)),
                    (0.8, LinearRgba::new(2.0, 2.0, 2.0, 0.2)),
                    (1.0, LinearRgba::new(0.0, 0.0, 0.0, 0.1)),
                ]),
            ),
        }
    }
}
