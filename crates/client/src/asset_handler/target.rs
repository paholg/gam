use bevy::prelude::{
    shape::{Circle, Cylinder},
    Color, Handle, Mesh, StandardMaterial,
};

use super::Builder;

pub struct TargetAssets {
    pub cursor_mesh: Handle<Mesh>,
    pub cursor_material: Handle<StandardMaterial>,
    pub laser_mesh: Handle<Mesh>,
    pub laser_material: Handle<StandardMaterial>,
    pub laser_length: f32,
}

impl TargetAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let target_material = StandardMaterial {
            emissive: Color::rgb_linear(10.0, 0.0, 0.1),
            ..Default::default()
        };

        let target_laser_material = StandardMaterial {
            emissive: Color::rgb_linear(10.0, 0.0, 0.1),
            ..Default::default()
        };
        let laser_length = 100.0;
        TargetAssets {
            cursor_mesh: builder.meshes.add(Circle::new(0.06).into()),
            cursor_material: builder.materials.add(target_material),
            laser_mesh: builder.meshes.add(
                Cylinder {
                    radius: 0.01,
                    height: 1.0,
                    resolution: 3,
                    segments: 1,
                }
                .into(),
            ),
            laser_material: builder.materials.add(target_laser_material),
            laser_length,
        }
    }
}
