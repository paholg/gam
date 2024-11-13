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
        let short_wall_color = Color::ALICE_BLUE;
        let wall_color = Color::AQUAMARINE;
        let tall_wall_color = Color::RED;

        let trans = |color: Color| StandardMaterial {
            base_color: color.with_a(0.5),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };

        WallAssets {
            shape: builder.meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            floor: builder.materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.6, 0.1),
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
