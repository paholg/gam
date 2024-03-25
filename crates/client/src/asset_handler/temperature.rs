use bevy::{
    math::primitives::Sphere,
    prelude::{AlphaMode, Color, Handle, Mesh, StandardMaterial},
};

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
            (0.0, Color::rgba(0.0, 20.0, 50.0, 0.3)),
            (0.5, Color::rgba(0.0, 0.0, 0.0, 0.0)),
            (1.0, Color::rgba(50.0, 20.0, 0.0, 0.3)),
        ]);
        let mesh = builder.meshes.add(Sphere::new(1.0));
        let material = builder.materials.add(StandardMaterial {
            base_color: Color::NONE,
            emissive: Color::NONE,
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
