use bevy::prelude::{shape::Icosphere, AlphaMode, Color, Handle, Mesh, StandardMaterial};

use crate::color_gradient::ColorGradient;

use super::Builder;

pub struct ExplosionAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    // TODO: Add a sound.
}

impl ExplosionAssets {
    pub fn new(builder: &mut Builder, colors: ColorGradient) -> Self {
        let initial_color = colors.get(0.0);
        ExplosionAssets {
            gradient: colors,
            mesh: builder.meshes.add(
                Mesh::try_from(Icosphere {
                    radius: 1.0,
                    subdivisions: 5,
                })
                .unwrap(),
            ),
            material: builder.materials.add(StandardMaterial {
                base_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
                emissive: initial_color,
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            }),
        }
    }
}
