use bevy::{
    math::primitives::Sphere,
    prelude::{AlphaMode, Color, Handle, Mesh, StandardMaterial},
};

use crate::shapes::HollowPolygon;

use super::Builder;

pub struct NeutrinoBallAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
}

impl NeutrinoBallAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let neutrino_ball_material = StandardMaterial {
            base_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };

        NeutrinoBallAssets {
            mesh: builder
                .meshes
                .add(Sphere::new(builder.props.neutrino_ball.radius)),
            material: builder.materials.add(neutrino_ball_material.clone()),
            outline_mesh: builder.meshes.add(HollowPolygon {
                radius: builder.props.neutrino_ball.effect_radius,
                thickness: 0.06,
                vertices: 60,
            }),
            outline_material: builder.materials.add(neutrino_ball_material),
        }
    }
}
