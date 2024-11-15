use bevy::color::LinearRgba;
use bevy::math::primitives::Cylinder;
use bevy::prelude::AlphaMode;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;

use super::Builder;
use crate::color_gradient::ColorGradient;

pub struct TransportAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl TransportAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let gradient = ColorGradient::new([
            (0.0, LinearRgba::new(0.0, 0.0, 2.0, 0.1)),
            (0.5, LinearRgba::new(0.0, 1.0, 2.0, 0.4)),
            (0.8, LinearRgba::new(0.0, 5.0, 5.0, 0.4)),
            (1.0, LinearRgba::new(0.0, 100.0, 100.0, 0.6)),
        ]);
        let base_color = gradient.get(0.0);
        Self {
            gradient,
            mesh: builder.meshes.add(Cylinder::new(1.0, 1.0)),
            material: builder.materials.add(StandardMaterial {
                base_color: base_color.into(),
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            }),
        }
    }
}
