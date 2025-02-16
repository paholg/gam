use std::f32::consts::PI;

use bevy::core_pipeline::bloom::Bloom;
use bevy::math::primitives::Rectangle;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::Assets;
use bevy::prelude::Camera;
use bevy::prelude::Camera3d;
use bevy::prelude::Color;
use bevy::prelude::Commands;
use bevy::prelude::Mesh;
use bevy::prelude::Mesh3d;
use bevy::prelude::PerspectiveProjection;
use bevy::prelude::Projection;
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
