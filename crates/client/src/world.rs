use std::f32::consts::PI;

use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::{
        shape, Assets, Camera, Camera3dBundle, Color, Commands, Mesh, PbrBundle,
        PerspectiveProjection, PointLight, PointLightBundle, ResMut, StandardMaterial, Transform,
        Vec2, Vec3,
    },
};
use engine::PLANE;

use crate::CAMERA_OFFSET;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(PLANE, PLANE),
                ..Default::default()
            }
            .into(),
        ),
        material: materials.add(Color::YELLOW_GREEN.into()),
        transform: Transform::from_xyz(0.0, 0.0, -0.1),
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

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(-0.5 * PLANE, -0.5 * PLANE, 10.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.5 * PLANE, -0.5 * PLANE, 10.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(-0.5 * PLANE, 0.5 * PLANE, 10.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.5 * PLANE, 0.5 * PLANE, 10.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..Default::default()
    });
}
