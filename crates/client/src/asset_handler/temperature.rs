use bevy::prelude::{shape::Icosphere, AlphaMode, Color, Handle, Mesh, StandardMaterial};

use crate::color_gradient::ColorGradient;

use super::Builder;

pub struct TemperatureAssets {
    pub gradient: ColorGradient,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

impl TemperatureAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let gradient = ColorGradient::new([
            (0.0, Color::rgba(0.0, 0.0, 10.0, 0.5)),
            (0.25, Color::rgba(0.0, 2.0, 5.0, 0.25)),
            (0.5, Color::rgba(0.0, 0.0, 0.0, 0.0)),
            (0.75, Color::rgba(5.0, 2.0, 0.0, 0.25)),
            (1.0, Color::rgba(10.0, 0.0, 0.0, 0.5)),
        ]);
        let mesh = builder.meshes.add(
            Mesh::try_from(Icosphere {
                radius: 1.0,
                subdivisions: 6,
            })
            .unwrap(),
        );
        let material = builder.materials.add(StandardMaterial {
            emissive: Color::NONE,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        });

        Self {
            gradient,
            mesh,
            material,
        }
    }
}
