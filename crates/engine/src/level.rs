use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, Resource},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{Collider, Friction, RigidBody};
use bevy_transform::components::{GlobalTransform, Transform};
use rand::Rng;

use crate::PLAYER_R;

/// A market to indicate that an entity is part of a level, and should be
/// deleted when it ends.
#[derive(Component, Default)]
pub struct InLevel;

pub fn clear_level(mut commands: Commands, query: Query<Entity, With<InLevel>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

const WALL_HEIGHT: f32 = 2.0;
const WALL_WIDTH: f32 = 0.5;
// const SHORT_WALL: f32 = 1.0;

#[derive(Resource)]
pub struct LevelProps {
    pub x: f32,
    pub y: f32,
}

impl Default for LevelProps {
    fn default() -> Self {
        Self { x: 50.0, y: 50.0 }
    }
}

impl LevelProps {
    pub fn point_in_plane(&self) -> Vec3 {
        let mut rng = rand::thread_rng();
        let x = rng.gen::<f32>() * (self.x - PLAYER_R) - (self.x - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (self.y - PLAYER_R) - (self.y - PLAYER_R) * 0.5;
        Vec3::new(x, y, 0.0)
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

pub fn default_level(mut commands: Commands, props: Res<LevelProps>) {
    let height = WALL_HEIGHT;
    let width = WALL_WIDTH;
    let commands = &mut commands;
    // Floor
    FloorSpawner::new(
        Vec3::new(props.x + width, props.y + width, height),
        Vec3::new(0.0, 0.0, -height * 0.5),
    )
    .spawn(commands);

    // Walls
    let half_wall = height * 0.5;
    FloorSpawner::new(
        Vec3::new(props.x + width, width, height),
        Vec3::new(0.0, -props.y * 0.5, half_wall),
    )
    .spawn(commands);

    FloorSpawner::new(
        Vec3::new(props.x + width, width, height),
        Vec3::new(0.0, props.y * 0.5, half_wall),
    )
    .spawn(commands);

    FloorSpawner::new(
        Vec3::new(width, props.y + width, height),
        Vec3::new(-props.x * 0.5, 0.0, half_wall),
    )
    .spawn(commands);

    FloorSpawner::new(
        Vec3::new(width, props.y + width, height),
        Vec3::new(props.x * 0.5, 0.0, half_wall),
    )
    .spawn(commands);
}
