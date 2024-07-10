use bevy::{
    asset::Assets,
    math::primitives::Sphere,
    prelude::{AlphaMode, Color, Handle, Mesh, StandardMaterial},
};

use crate::color_gradient::ColorGradient;

pub struct ExplosionAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    // TODO: Add a sound.
}

impl ExplosionAssets {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        colors: ColorGradient,
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
