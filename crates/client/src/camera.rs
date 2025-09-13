use std::f32::consts::PI;

use bevy::{
    animation::animatable::Animatable,
    app::{Plugin, Startup, Update},
    core_pipeline::{bloom::Bloom, core_3d::Camera3d},
    ecs::{
        component::Component,
        error::Result,
        query::{QueryData, With, Without},
        schedule::IntoScheduleConfigs,
        system::{Commands, Query, Res},
    },
    input::mouse::AccumulatedMouseMotion,
    math::{Dir3, EulerRot, Quat, Ray3d, Vec3},
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings},
    render::camera::{Camera, PerspectiveProjection, Projection},
    time::Time,
    transform::components::Transform,
};
use engine::{game_running, Player, Target, FORWARD, UP};

use crate::{aim::BlocksSight, draw::level::WallKind};

pub struct CameraPlugin;

#[derive(Component, Default)]
struct CameraAngles {
    pitch: f32,
    yaw: f32,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, update.run_if(game_running));
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: PI * 0.25,
            ..Default::default()
        }),
        Transform::from_translation(Vec3::new(0.0, 12.0, 12.0)).looking_at(Vec3::ZERO, UP),
        Bloom::default(),
        CameraAngles::default(),
    ));
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PlayerQuery {
    // FIXME: We shouldn't rotate the player here.
    transform: &'static mut Transform,
    target: &'static mut Target,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CameraQuery {
    transform: &'static mut Transform,
    angles: &'static mut CameraAngles,
}

fn update(
    mut player_query: Query<PlayerQuery, With<Player>>,
    mut camera_query: Query<CameraQuery, Without<Player>>,
    cursor_motion: Res<AccumulatedMouseMotion>,
    mut raycast: MeshRayCast,
    world_query: Query<(), With<WallKind>>,
    blocks_sight_query: Query<(), With<BlocksSight>>,
    time: Res<Time>,
) -> Result {
    let Ok(mut player) = player_query.single_mut() else {
        return Ok(()); // RIP
    };
    let mut camera = camera_query.single_mut()?;

    // Constants / Settings
    const TOP_MAX_PITCH: f32 = PI / 2.1;
    const BOTTOM_MAX_PITCH: f32 = PI / 2.1;

    let camera_offset = Vec3::new(0.0, 1.5, 8.0);
    let sensitivity = 0.001;
    let lerp_factor: f32 = 0.001;

    // Set angles
    let (_, player_pitch, player_roll) = player.transform.rotation.to_euler(EulerRot::default());

    let cursor_delta = cursor_motion.delta * sensitivity;

    camera.angles.yaw -= cursor_delta.x;
    camera.angles.pitch -= cursor_delta.y;
    camera.angles.pitch = camera.angles.pitch.clamp(
        player_pitch - BOTTOM_MAX_PITCH,
        player_pitch + TOP_MAX_PITCH,
    );

    let target_quat = Quat::from_euler(
        EulerRot::default(),
        camera.angles.yaw,
        camera.angles.pitch,
        player_roll,
    );
    let target_dir = Dir3::new(target_quat * FORWARD).unwrap();

    // Target raycast
    {
        let filter = |entity| blocks_sight_query.get(entity).is_ok();
        let settings = MeshRayCastSettings::default()
            .with_filter(&filter)
            .with_visibility(bevy::picking::mesh_picking::ray_cast::RayCastVisibility::Visible);

        let ray = Ray3d::new(player.transform.translation, target_dir);

        let hit = raycast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_entity, hit)| hit);
        player.target.transform.translation = hit
            .map(|h| h.point)
            .unwrap_or_else(|| ray.get_point(1000.0));
        player.target.transform.rotation = target_quat;
    }

    {
        // FIXME: Don't do this here!
        player.transform.rotation = Quat::from_euler(
            EulerRot::default(),
            camera.angles.yaw,
            player_pitch,
            player_roll,
        );
    }

    // Position camera
    {
        let filter = |entity| world_query.get(entity).is_ok();
        let settings = MeshRayCastSettings::default()
            .with_filter(&filter)
            .with_visibility(bevy::picking::mesh_picking::ray_cast::RayCastVisibility::Visible);

        let desired_camera_translation = player.transform.translation + target_quat * camera_offset;

        let dir = Dir3::new(desired_camera_translation - player.transform.translation).unwrap();
        let ray = Ray3d::new(player.transform.translation, dir);

        let point = raycast
            .cast_ray(ray, &settings)
            .first()
            .map(|(_, hit)| hit.point)
            .unwrap_or(desired_camera_translation);

        let final_camera_transform = Transform::from_translation(point).with_rotation(target_quat);

        let dt = time.delta_secs();
        let t = 1.0 - lerp_factor.powf(dt);
        *camera.transform = Transform::interpolate(&camera.transform, &final_camera_transform, t);
    }

    Ok(())
}
