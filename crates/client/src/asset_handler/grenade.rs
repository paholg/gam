use bevy::{
    math::primitives::Sphere,
    prelude::{Color, Handle, Mesh, StandardMaterial},
};

use crate::{color_gradient::ColorGradient, shapes::HollowPolygon};

use super::{explosion::ExplosionAssets, Builder};

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
            emissive: Color::rgb_linear(10.0, 0.0, 0.1),
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
                    (0.0, Color::rgba(5.0, 1.2, 0.0, 0.2)),
                    (0.5, Color::rgba(10.0, 2.5, 0.0, 0.2)),
                    (0.8, Color::rgba(0.2, 0.2, 0.2, 0.2)),
                    (1.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
                ]),
            ),
        }
    }

    pub fn heal(builder: &mut Builder) -> Self {
        let heal_grenade_material = StandardMaterial {
            emissive: Color::rgb_linear(0.0, 10.0, 0.1),
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
                    (0.0, Color::rgba(0.0, 5.0, 0.0, 0.2)),
                    (0.5, Color::rgba(0.0, 10.0, 0.0, 0.2)),
                    (0.8, Color::rgba(0.2, 0.2, 0.2, 0.2)),
                    (1.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
                ]),
            ),
        }
    }
}
