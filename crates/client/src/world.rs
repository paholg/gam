use bevy::{
    math::primitives::Rectangle,
    pbr::MeshMaterial3d,
    prelude::{Assets, Color, Commands, Mesh, Mesh3d, ResMut, StandardMaterial, Vec3},
};
use engine::lifecycle::DEATH_Y;

use crate::in_plane;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Death floor
    commands.spawn((
        Mesh3d::from(meshes.add(Rectangle::new(10_000.0, 10_000.0))),
        MeshMaterial3d::from(materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..Default::default()
        })),
        in_plane().with_translation(Vec3::new(0.0, DEATH_Y, 0.0)),
    ));
}
