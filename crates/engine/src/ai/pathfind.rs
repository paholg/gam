use std::sync::{Arc, RwLock};

use bevy_app::Plugin;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader},
    query::{QueryData, Without},
    system::{Commands, Query, Res, ResMut, Resource},
    world::Mut,
};
use bevy_math::{Vec2, Vec3};
use bevy_rapier3d::prelude::Collider;
use bevy_tasks::{AsyncComputeTaskPool, Task};
use bevy_transform::components::Transform;
use futures_lite::future;
use oxidized_navigation::{
    query::find_path, tiles::NavMeshTiles, NavMesh, NavMeshSettings, OxidizedNavigationPlugin,
};

use crate::{
    level::LevelProps, lifecycle::DEATH_Y, movement::DesiredMove, FootOffset, To2d, To3d, PLAYER_R,
};

use super::AiTarget;

pub struct PathfindPlugin;

impl Plugin for PathfindPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let level_props: &LevelProps = app.world.get_resource().unwrap();
        let extents = level_props.x.max(level_props.z);

        app.add_plugins(OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings {
            cell_width: PLAYER_R * 0.5,
            cell_height: 0.06,
            tile_width: 100,
            world_half_extents: extents * 0.5,
            world_bottom_bound: DEATH_Y,
            max_traversable_slope_radians: 1.0,
            walkable_height: u16::MAX,
            walkable_radius: 2,
            step_height: 1,
            min_region_area: 100,
            merge_region_area: 500,
            max_edge_length: 80,
            max_contour_simplification_error: 1.1,
            max_tile_generation_tasks: Some(1),
        }))
        .insert_resource(PathfindingTasks::default())
        .add_event::<PathfindEvent>();
    }
}

#[derive(Event)]
pub struct PathfindEvent {
    pub entity: Entity,
    pub target: Vec2,
}

#[derive(Component)]
pub struct Pathfinding;

#[derive(Component, Default)]
pub struct HasPath {
    pub path: Vec<Vec3>,
}

pub struct FoundPath {
    pub entity: Entity,
    pub path: Option<Vec<Vec3>>,
}

impl FoundPath {
    fn new(entity: Entity, path: Vec<Vec3>) -> Self {
        Self {
            entity,
            path: Some(path),
        }
    }

    fn without_path(entity: Entity) -> Self {
        Self { entity, path: None }
    }
}

#[derive(Default, Resource)]
pub struct PathfindingTasks {
    pub tasks: Vec<Task<FoundPath>>,
}

#[derive(QueryData)]
pub struct PathfindingQuery {
    transform: &'static Transform,
    foot_offset: &'static FootOffset,
    target: &'static AiTarget,
}

pub fn pathfinding_system(
    mut commands: Commands,
    nav_mesh_settings: Res<NavMeshSettings>,
    nav_mesh: Res<NavMesh>,
    mut pathfinding_tasks: ResMut<PathfindingTasks>,
    mut events: EventReader<PathfindEvent>,
    ai_q: Query<PathfindingQuery, Without<Pathfinding>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let nav_mesh_lock = nav_mesh.get();

    for event in events.read() {
        let Ok(ai) = ai_q.get(event.entity) else {
            continue;
        };

        let start = ai.transform.translation + ai.foot_offset.to_vec();
        let end = event.target.to_3d(0.0);

        let task = thread_pool.spawn(async_path_find(
            nav_mesh_lock.clone(),
            nav_mesh_settings.clone(),
            event.entity,
            start,
            end,
        ));
        pathfinding_tasks.tasks.push(task);

        commands.entity(event.entity).insert(Pathfinding);
    }
}

async fn async_path_find(
    nav_mesh_lock: Arc<RwLock<NavMeshTiles>>,
    nav_mesh_settings: NavMeshSettings,
    entity: Entity,
    start: Vec3,
    end: Vec3,
) -> FoundPath {
    let Ok(nav_mesh) = nav_mesh_lock.read() else {
        tracing::warn!("Could not read nav_mesh");
        return FoundPath::without_path(entity);
    };

    match find_path(
        &nav_mesh,
        &nav_mesh_settings,
        start,
        end,
        None,
        Some(&[1.0, 0.5]),
    ) {
        Ok(path) => FoundPath::new(entity, path),
        Err(error) => {
            tracing::warn!(?error, "Pathfinding error");
            FoundPath::without_path(entity)
        }
    }
}

pub fn poll_pathfinding_system(mut commands: Commands, mut tasks: ResMut<PathfindingTasks>) {
    tasks
        .tasks
        .retain_mut(|task| match future::block_on(future::poll_once(task)) {
            Some(found_path) => {
                // This job may have been started on a previous frame, so the
                // entity may no longer exist.
                if let Some(mut ecmds) = commands.get_entity(found_path.entity) {
                    ecmds.remove::<Pathfinding>();
                    match found_path.path {
                        Some(path) => {
                            ecmds.insert(HasPath { path });
                        }
                        None => {
                            // Job done, but no path found. We'll try again.
                        }
                    }
                }
                false
            }
            None => true,
        });
}

const CLOSE_ENOUGH: f32 = 0.3;

// TODO: Smooth out movement, so we're not making sharp, robotic turns along
// the path.
pub fn set_move(
    mut path: Mut<'_, HasPath>,
    transform: &Transform,
    mut desired_move: Mut<'_, DesiredMove>,
) -> Option<Vec2> {
    let dest = path.path.first()?.to_2d();

    if transform.translation.to_2d().distance_squared(dest) < CLOSE_ENOUGH * CLOSE_ENOUGH {
        // We're close enough
        path.path.remove(0);
    }

    let dest = path.path.first()?.to_2d();

    // TODO: We should maybe return < 1.0 when the situation calls for it, so we
    // slow down around corners. Or maybe not.
    desired_move.dir = (dest - transform.translation.to_2d()).normalize_or_zero();
    Some(desired_move.dir)
}
