use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{Collider, Friction, RigidBody};
use bevy_transform::components::{GlobalTransform, Transform};
use rand::Rng;

use crate::{lifecycle::DEATH_Y, PLAYER_R};

/// A market to indicate that an entity is part of a level, and should be
/// deleted when it ends.
#[derive(Component, Default)]
pub struct InLevel;

pub fn clear_level(mut commands: Commands, query: Query<Entity, With<InLevel>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

pub const WALL_HEIGHT: f32 = 2.0;
const WALL_WIDTH: f32 = 0.5;
pub const SHORT_WALL: f32 = 1.0;

#[derive(Resource)]
pub struct LevelProps {
    pub x: f32,
    pub z: f32,
}

impl Default for LevelProps {
    fn default() -> Self {
        Self { x: 50.0, z: 50.0 }
    }
}

impl LevelProps {
    pub fn point_in_plane(&self) -> Vec3 {
        let mut rng = rand::thread_rng();
        let x = rng.gen::<f32>() * (self.x - PLAYER_R) - (self.x - PLAYER_R) * 0.5;
        let z = rng.gen::<f32>() * (self.z - PLAYER_R) - (self.z - PLAYER_R) * 0.5;
        Vec3::new(x, 0.0, z)
    }
}

struct FloorSpawner {
    dim: Vec3,
    loc: Vec3,
}

#[derive(Component)]
pub struct Floor {
    pub dim: Vec3,
    pub loc: Vec3,
}

impl FloorSpawner {
    fn new(dim: Vec3, loc: Vec3) -> Self {
        Self { dim, loc }
    }

    fn spawn(self, commands: &mut Commands) {
        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(self.dim.x * 0.5, self.dim.y * 0.5, self.dim.z * 0.5),
            Transform::from_translation(self.loc),
            GlobalTransform::default(),
            Friction::default(),
            InLevel,
            Floor {
                dim: self.dim,
                loc: self.loc,
            },
        ));
    }
}

const PIT_COLOR: [u8; 3] = [0, 0, 0];
const FLOOR_COLOR: [u8; 3] = [150, 240, 110];
const WALL_COLOR: [u8; 3] = [220, 110, 165];
const SHORT_WALL_COLOR: [u8; 3] = [255, 200, 255];

pub fn test_level(mut commands: Commands, mut props: ResMut<LevelProps>) {
    let pixel = 1.0;
    let image = image::io::Reader::open("assets/levels/test2.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb8();
    let (width, height) = image.dimensions();
    props.x = width as f32 * pixel;
    props.z = height as f32 * pixel;

    for (x, z, color) in image.enumerate_pixels() {
        let height = match color.0 {
            PIT_COLOR => continue,
            FLOOR_COLOR => 0.0,
            SHORT_WALL_COLOR => SHORT_WALL,
            WALL_COLOR => WALL_HEIGHT,
            color => {
                tracing::warn!("Not an acceptable color: {color:?}");
                WALL_HEIGHT * 2.0
            }
        };
        let x = x as f32 * pixel;
        let z = z as f32 * pixel;
        let dim = Vec3::new(pixel, -DEATH_Y + height, pixel);
        let loc = Vec3::new(
            -props.x * 0.5 + x,
            DEATH_Y * 0.5 + height * 0.5,
            -props.z * 0.5 + z,
        );
        FloorSpawner::new(dim, loc).spawn(&mut commands);
    }
}

pub fn default_level(mut commands: Commands, props: Res<LevelProps>) {
    let height = WALL_HEIGHT;
    let width = WALL_WIDTH;
    let commands = &mut commands;
    // Floor
    FloorSpawner::new(
        Vec3::new(props.x + width, height, props.z + height),
        Vec3::new(0.0, -height * 0.5, 0.0),
    )
    .spawn(commands);

    // Walls
    let half_wall = height * 0.5;
    FloorSpawner::new(
        Vec3::new(props.x + width, height, width),
        Vec3::new(0.0, half_wall, -props.z * 0.5),
    )
    .spawn(commands);

    FloorSpawner::new(
        Vec3::new(props.x + width, width, height),
        Vec3::new(0.0, half_wall, props.z * 0.5),
    )
    .spawn(commands);

    FloorSpawner::new(
        Vec3::new(width, height, props.z + width),
        Vec3::new(-props.x * 0.5, half_wall, 0.0),
    )
    .spawn(commands);

    FloorSpawner::new(
        Vec3::new(width, height, props.z + width),
        Vec3::new(props.x * 0.5, half_wall, 0.0),
    )
    .spawn(commands);
}
