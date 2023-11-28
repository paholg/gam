use std::f32::consts::PI;

use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::{
        Camera, Camera3dBundle, Commands, PerspectiveProjection, PointLight, PointLightBundle, Res,
        Transform, Vec3,
    },
};
use engine::level::{InLevel, LevelProps};

use crate::CAMERA_OFFSET;

pub fn setup(mut commands: Commands, level: Res<LevelProps>) {
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

    let light_range = (level.x + level.y) * 0.5;

    // Light
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                range: light_range,
                intensity: 1000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(-0.5 * level.x, -0.5 * level.y, 10.0),
            ..Default::default()
        },
        InLevel,
    ));
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                range: light_range,
                intensity: 1000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.5 * level.x, -0.5 * level.y, 10.0),
            ..Default::default()
        },
        InLevel,
    ));
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                range: light_range,
                intensity: 1000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(-0.5 * level.x, 0.5 * level.y, 10.0),
            ..Default::default()
        },
        InLevel,
    ));
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                range: light_range,
                intensity: 1000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.5 * level.x, 0.5 * level.y, 10.0),
            ..Default::default()
        },
        InLevel,
    ));
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                range: light_range,
                intensity: 1000.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..Default::default()
        },
        InLevel,
    ));
}
