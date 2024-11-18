use bevy::color::palettes::css::BLACK;
use bevy::color::palettes::css::GREEN;
use bevy::math::primitives::Rectangle;
use bevy::prelude::Color;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;

use super::Builder;

pub struct BarAssets {
    pub mesh: Handle<Mesh>,
    pub fg_material: Handle<StandardMaterial>,
    pub bg_material: Handle<StandardMaterial>,
}

// NOTE: We use a very large value for depth_bias, because it doesn't play
// nicely with scale otherwise. If it's, say "1.0", and we have a small value
// for scale, it doesn't seem to help.
impl BarAssets {
    pub fn healthbar(builder: &mut Builder) -> Self {
        let fg = StandardMaterial {
            base_color: GREEN.into(),
            unlit: true,
            depth_bias: 1000.0,
            ..Default::default()
        };
        let bg = StandardMaterial {
            base_color: BLACK.into(),
            unlit: true,
            depth_bias: -1000.0,
            ..Default::default()
        };
        BarAssets {
            mesh: builder.meshes.add(Rectangle::new(1.0, 1.0)),
            fg_material: builder.materials.add(fg),
            bg_material: builder.materials.add(bg),
        }
    }

    pub fn energybar(builder: &mut Builder) -> Self {
        let fg = StandardMaterial {
            base_color: Color::linear_rgb(0.0, 0.2, 0.8),
            unlit: true,
            depth_bias: 1000.0,
            ..Default::default()
        };
        let bg = StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            depth_bias: -1000.0,
            ..Default::default()
        };
        BarAssets {
            mesh: builder.meshes.add(Rectangle::new(1.0, 1.0)),
            fg_material: builder.materials.add(fg),
            bg_material: builder.materials.add(bg),
        }
    }
}
