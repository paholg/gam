use bevy::color::LinearRgba;
use bevy::math::primitives::Capsule3d;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;

use super::explosion::ExplosionAssets;
use super::Builder;
use crate::color_gradient::ColorGradient;

pub struct SeekerRocketAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub explosion: ExplosionAssets,
}

impl SeekerRocketAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let seeker_rocket_material = StandardMaterial {
            emissive: LinearRgba::rgb(10_000.0, 100.0, 10_000.0),
            ..Default::default()
        };

        SeekerRocketAssets {
            mesh: builder.meshes.add(Capsule3d::new(
                builder.props.seeker_rocket.capsule_radius,
                builder.props.seeker_rocket.capsule_length,
            )),
            material: builder.materials.add(seeker_rocket_material.clone()),
            explosion: ExplosionAssets::new(
                builder,
                ColorGradient::new([
                    (0.0, LinearRgba::new(50.0, 1.0, 50.0, 0.2)),
                    (0.5, LinearRgba::new(100.0, 1.0, 100.0, 0.2)),
                    (0.8, LinearRgba::new(2.0, 2.0, 2.0, 0.2)),
                    (1.0, LinearRgba::new(0.0, 0.0, 0.0, 0.1)),
                ]),
            ),
        }
    }
}
