use bevy::{
    color::{
        palettes::css::{BLACK, GREEN},
        LinearRgba,
    },
    math::primitives::Rectangle,
    prelude::{Color, Handle, Mesh, StandardMaterial},
};

use super::Builder;

pub struct BarAssets {
    pub mesh: Handle<Mesh>,
    pub fg_material: Handle<StandardMaterial>,
    pub bg_material: Handle<StandardMaterial>,
}

impl BarAssets {
    pub fn healthbar(builder: &mut Builder) -> Self {
        let fg = StandardMaterial {
            base_color: GREEN.into(),
            unlit: true,
            ..Default::default()
        };
        let bg = StandardMaterial {
            base_color: BLACK.into(),
            unlit: true,
            ..Default::default()
        };
        BarAssets {
            mesh: builder.meshes.add(Rectangle::new(1.0, 1.0)),
            fg_material: builder.materials.add(fg),
            bg_material: builder.materials.add(bg.clone()),
        }
    }

    pub fn energybar(builder: &mut Builder) -> Self {
        let fg = StandardMaterial {
            base_color: LinearRgba {
                red: 0.0,
                green: 0.2,
                blue: 0.8,
                alpha: 1.0,
            }
            .into(),
            unlit: true,
            ..Default::default()
        };
        let bg = StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..Default::default()
        };
        BarAssets {
            mesh: builder.meshes.add(Rectangle::new(1.0, 1.0)),
            fg_material: builder.materials.add(fg),
            bg_material: builder.materials.add(bg.clone()),
        }
    }
}
