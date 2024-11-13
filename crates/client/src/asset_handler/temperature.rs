use bevy::color::LinearRgba;
use bevy::math::primitives::Sphere;
use bevy::prelude::AlphaMode;
use bevy::prelude::Color;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;

use super::Builder;
use crate::color_gradient::ColorGradient;

pub struct TemperatureAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl TemperatureAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let gradient = ColorGradient::new([
            (0.0, LinearRgba::new(0.0, 20.0, 50.0, 0.3)),
            (0.5, LinearRgba::new(0.0, 0.0, 0.0, 0.0)),
            (1.0, LinearRgba::new(50.0, 20.0, 0.0, 0.3)),
        ]);
        let mesh = builder.meshes.add(Sphere::new(1.0));
        let material = builder.materials.add(StandardMaterial {
            base_color: Color::NONE,
            emissive: LinearRgba::NONE,
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..Default::default()
        });

        Self {
            gradient,
            mesh,
            material,
        }
    }
}
