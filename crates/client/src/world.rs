use std::f32::consts::PI;

use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::{
        shape, Assets, Camera, Camera3dBundle, Color, Commands, Mesh, PbrBundle,
        PerspectiveProjection, ResMut, StandardMaterial, Transform, Vec2, Vec3,
    },
};
use engine::lifecycle::DEATH_Z;

use crate::CAMERA_OFFSET;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Death floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(100_000.0, 100_000.0),
                ..Default::default()
            }
            .into(),
        ),
        material: materials.add(Color::BLACK.into()),
        transform: Transform::from_xyz(0.0, 0.0, DEATH_Z),
        ..Default::default()
    });

    // Camera
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            projection: PerspectiveProjection {
                fov: PI * 0.125,
                ..Default::default()
            }
            .into(),
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
            ..Default::default()
        },
        BloomSettings::default(),
    ));
}
