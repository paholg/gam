use bevy::prelude::{shape::Capsule, Color, Handle, Mesh, StandardMaterial};

use crate::color_gradient::ColorGradient;

use super::{explosion::ExplosionAssets, Builder};

pub struct SeekerRocketAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
}

impl SeekerRocketAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let seeker_rocket_material = StandardMaterial {
            emissive: Color::rgb_linear(10.0, 0.1, 10.0),
            ..Default::default()
        };

        SeekerRocketAssets {
            mesh: builder.meshes.add(
                Mesh::try_from(Capsule {
                    radius: builder.props.seeker_rocket.capsule_radius,
                    depth: builder.props.seeker_rocket.capsule_length,
                    ..Default::default()
                })
                .unwrap(),
            ),
            material: builder.materials.add(seeker_rocket_material.clone()),
            explosion: ExplosionAssets::new(
                builder,
                ColorGradient::new([
                    (0.0, Color::rgba(5.0, 0.1, 5.0, 0.2)),
                    (0.5, Color::rgba(10.0, 0.1, 10.0, 0.2)),
                    (0.8, Color::rgba(0.2, 0.2, 0.2, 0.2)),
                    (1.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
                ]),
            ),
        }
    }
}
