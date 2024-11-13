use std::f32::consts::PI;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::math::primitives::Rectangle;
use bevy::prelude::Assets;
use bevy::prelude::Camera;
use bevy::prelude::Camera3dBundle;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Mesh;
use bevy::prelude::PbrBundle;
use bevy::prelude::PerspectiveProjection;
use bevy::prelude::ResMut;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use engine::lifecycle::DEATH_Y;
use engine::UP;

use crate::in_plane;
use crate::CAMERA_OFFSET;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Death floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(Rectangle::new(10_000.0, 10_000.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..Default::default()
        }),
        transform: in_plane().with_translation(Vec3::new(0.0, DEATH_Y, 0.0)),
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
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, UP),
            ..Default::default()
        },
        BloomSettings::default(),
    ));
}
