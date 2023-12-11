use bevy::prelude::{shape::Cylinder, AlphaMode, Color, Handle, Mesh, StandardMaterial};

use crate::color_gradient::ColorGradient;

use super::Builder;

pub struct TransportAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl TransportAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let gradient = ColorGradient::new([
            (0.0, Color::rgba(0.0, 0.0, 1.0, 0.1)),
            (0.5, Color::rgba(0.0, 0.5, 1.0, 0.4)),
            (0.8, Color::rgba(0.0, 1.0, 1.0, 0.8)),
            (1.0, Color::rgba(0.0, 10.0, 10.0, 0.8)),
        ]);
        let base_color = gradient.get(0.0);
        Self {
            gradient,
            mesh: builder.meshes.add(
                Mesh::try_from(Cylinder {
                    radius: 1.0,
                    height: 1.0,
                    ..Default::default()
                })
                .unwrap(),
            ),
            material: builder.materials.add(StandardMaterial {
                base_color,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            }),
        }
    }
}