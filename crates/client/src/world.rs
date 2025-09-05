use std::f32::consts::PI;

use bevy::{
    core_pipeline::bloom::Bloom,
    math::primitives::Rectangle,
    pbr::MeshMaterial3d,
    prelude::{
        Assets, Camera, Camera3d, Color, Commands, Mesh, Mesh3d, PerspectiveProjection, Projection,
        ResMut, StandardMaterial, Transform, Vec3,
    },
};
use engine::{lifecycle::DEATH_Y, UP};

use crate::{in_plane, CAMERA_OFFSET};

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

    // Camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: PI * 0.125,
            ..Default::default()
        }),
        Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, UP),
        Bloom::default(),
    ));
}
