use bevy::color::palettes::css::ALICE_BLUE;
use bevy::color::palettes::css::AQUAMARINE;
use bevy::color::palettes::css::RED;
use bevy::color::Alpha;
use bevy::math::primitives::Cuboid;
use bevy::prelude::AlphaMode;
use bevy::prelude::Color;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;

use super::Builder;

pub struct WallAssets {
    pub shape: Handle<Mesh>,
    pub floor: Handle<StandardMaterial>,
    pub short_wall: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub tall_wall: Handle<StandardMaterial>,
    pub short_wall_trans: Handle<StandardMaterial>,
    pub wall_trans: Handle<StandardMaterial>,
    pub tall_wall_trans: Handle<StandardMaterial>,
}

impl WallAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let short_wall_color = ALICE_BLUE.into();
        let wall_color = AQUAMARINE.into();
        let tall_wall_color = RED.into();

        let trans = |color: Color| StandardMaterial {
            base_color: color.with_alpha(0.5),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };

        WallAssets {
            shape: builder.meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            floor: builder.materials.add(StandardMaterial {
                base_color: Color::srgb(0.0, 0.6, 0.1),
                perceptual_roughness: 0.8,
                ..Default::default()
            }),
            short_wall: builder.materials.add(short_wall_color),
            wall: builder.materials.add(wall_color),
            tall_wall: builder.materials.add(tall_wall_color),
            short_wall_trans: builder.materials.add(trans(short_wall_color)),
            wall_trans: builder.materials.add(trans(wall_color)),
            tall_wall_trans: builder.materials.add(trans(tall_wall_color)),
        }
    }
}
