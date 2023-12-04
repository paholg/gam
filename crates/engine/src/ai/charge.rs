use std::sync::{Arc, RwLock};

use bevy_app::Plugin;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{With, Without},
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Commands, Query, Res, ResMut, Resource},
};
use bevy_math::{Vec2, Vec3};
use bevy_rapier3d::prelude::{Collider, QueryFilter, RapierContext, Velocity};
use bevy_tasks::{AsyncComputeTaskPool, Task};
use bevy_transform::components::Transform;
use futures_lite::future;
use oxidized_navigation::{
    query::find_path, tiles::NavMeshTiles, NavMesh, NavMeshSettings, OxidizedNavigationPlugin,
};

use crate::{
    ability::{properties::AbilityProps, Ability},
    level::{Floor, LevelProps},
    lifecycle::DEATH_Y,
    movement::DesiredMove,
    status_effect::StatusEffects,
    time::TickCounter,
    AbilityOffset, Ally, Cooldowns, Enemy, Energy, Faction, FootOffset, Target, To2d, To3d,
    PLAYER_R,
};

use super::{update_target_system, AiTarget};

pub struct ChargeAiPlugin;

impl Plugin for ChargeAiPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let level_props: &LevelProps = app.world.get_resource().unwrap();
        let extents = level_props.x.max(level_props.z);

        app.add_plugins((OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings {
            cell_width: PLAYER_R * 0.5,
            cell_height: 0.06,
            tile_width: 100,
            world_half_extents: extents * 0.5,
            world_bottom_bound: DEATH_Y,
            max_traversable_slope_radians: 1.0,
            walkable_height: 10,
            walkable_radius: 2,
            step_height: 1,
            min_region_area: 100,
            merge_region_area: 500,
            max_edge_length: 80,
            max_contour_simplification_error: 1.1,
            max_tile_generation_tasks: Some(1),
        }),))
            .insert_resource(PathfindingTasks::default());
    }
}

#[derive(Component)]
pub struct ChargeAi {
    pub desired_range: f32,
    /// The distance the target gets from the end of the path before we
    /// re-compute it.
    pub target_dist: f32,
}

impl Default for ChargeAi {
    fn default() -> Self {
        Self {
            desired_range: 1.0,
            target_dist: 3.0,
        }
    }
}

#[derive(Component)]
pub struct Pathfinding;

#[derive(Component)]
pub struct HasPath {
    pub path: Vec<Vec3>,
}

pub fn system_set() -> SystemConfigs {
    (
        update_target_system::<Enemy, ChargeAi>,
        update_target_system::<Ally, ChargeAi>,
        move_system,
        gun_system::<Enemy>,
        gun_system::<Ally>,
        async_pathfinding,
        poll_pathfinding_system,
    )
        .chain()
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
/// Compute pathfinding for any ChargeAi that we don't currently have
/// pathfinding for.
fn async_pathfinding(
    mut commands: Commands,
    nav_mesh_settings: Res<NavMeshSettings>,
    nav_mesh: Res<NavMesh>,
    mut pathfinding_tasks: ResMut<PathfindingTasks>,
    ai_q: Query<
        (
            Entity,
            &Transform,
            &FootOffset,
            &AiTarget,
            Option<&HasPath>,
            &ChargeAi,
        ),
        Without<Pathfinding>,
    >,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let nav_mesh_lock = nav_mesh.get();

    for (entity, transform, foot_offset, target, has_path, ai) in &ai_q {
        if let Some(path) = has_path {
            if let Some(last) = path.path.last() {
                if last.to_2d().distance_squared(target.loc.0) < ai.target_dist * ai.target_dist {
                    // We already have a good enough path.
                    continue;
                }
            }
        }
        let start = transform.translation + foot_offset.to_vec();
        let end = target.loc.0.to_3d(0.0);

        let task = thread_pool.spawn(async_path_find(
            nav_mesh_lock.clone(),
            nav_mesh_settings.clone(),
            entity,
            start,
            end,
        ));
        pathfinding_tasks.tasks.push(task);

        commands.entity(entity).insert(Pathfinding);
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

fn poll_pathfinding_system(mut commands: Commands, mut tasks: ResMut<PathfindingTasks>) {
    tasks
        .tasks
        .retain_mut(|task| match future::block_on(future::poll_once(task)) {
            Some(found_path) => {
                // This entity was computed on a previous frame; we have to be
                // careful to ensure that it still exists.
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

fn move_dir(path: &mut Vec<Vec3>, transform: &Transform, _velocity: &Velocity) -> Option<Vec2> {
    if path.is_empty() {
        return None;
    }

    let delta = (transform.translation - path[0]).to_2d();
    // We're close enough
    if delta.length_squared() < CLOSE_ENOUGH * CLOSE_ENOUGH {
        path.remove(0);
        if path.is_empty() {
            return None;
        }
    }

    Some((path[0].to_2d() - transform.translation.to_2d()).normalize_or_zero())
}

fn move_system(
    mut commands: Commands,
    mut ai_q: Query<(
        Entity,
        &Transform,
        &Velocity,
        &mut DesiredMove,
        &mut HasPath,
        &AiTarget,
        &ChargeAi,
    )>,
) {
    for (entity, transform, velocity, mut desired_move, mut path, target, ai) in &mut ai_q {
        let target_delta = target.loc.0 - transform.translation.to_2d();

        if target_delta.length_squared() < ai.desired_range * ai.desired_range {
            // We're already within desired range.
            // TODO: We'll want to keep moving if we don't have LOS on target.
            // TODO: We'll want to move _somewhere_ even when in range.
            continue;
        }

        match move_dir(&mut path.path, transform, velocity) {
            Some(dir) => desired_move.dir = dir,
            None => {
                // The path is empty; remove it so we can get another.
                commands.entity(entity).remove::<HasPath>();
            }
        }
    }
}

fn gun_system<T: Faction>(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut ai_q: Query<
        (
            Entity,
            &mut Cooldowns,
            &mut Energy,
            &Velocity,
            &Transform,
            &mut StatusEffects,
            &AbilityOffset,
            &AiTarget,
        ),
        (With<T>, With<ChargeAi>),
    >,
    props: Res<AbilityProps>,
    rapier_context: Res<RapierContext>,
    wall_q: Query<(), With<Floor>>,
    friend_q: Query<(), With<T>>,
) {
    // Filter for friends or walls; we don't want to shoot them. But we ignore
    // self, as the ray will start inside our collider.
    let exclude_walls_friends = |this, target, entity| {
        if entity == this || entity == target {
            false
        } else {
            wall_q.get(entity).is_ok() || friend_q.get(entity).is_ok()
        }
    };
    for (
        entity,
        mut cooldowns,
        mut energy,
        velocity,
        transform,
        mut status_effects,
        ability_offset,
        target,
    ) in ai_q.iter_mut()
    {
        let Some(target_entity) = target.entity else {
            continue;
        };
        let pred = |e| exclude_walls_friends(entity, target_entity, e);
        let filter = QueryFilter::new().predicate(&pred);
        let origin = transform.translation + ability_offset.to_vec();

        let dir = target.loc.0 - transform.translation.to_2d();

        let ray = rapier_context.cast_ray(origin, dir.to_3d(0.0), 1.0, true, filter);
        if ray.is_none() {
            Ability::Gun.fire(
                &mut commands,
                &tick_counter,
                &props,
                entity,
                &mut energy,
                &mut cooldowns,
                transform,
                velocity,
                &mut status_effects,
                &Target::default(),
                ability_offset,
            );
        }
    }
}
