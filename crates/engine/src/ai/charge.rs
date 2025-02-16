use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::QueryData;
use bevy_ecs::query::With;
use bevy_ecs::query::Without;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::schedule::SystemConfigs;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_math::Vec2;
use bevy_rapier3d::plugin::ReadDefaultRapierContext;
use bevy_rapier3d::prelude::QueryFilter;
use bevy_transform::components::Transform;
use rand::Rng;

use super::pathfind::set_move;
use super::pathfind::HasPath;
use super::pathfind::PathfindEvent;
use super::target_closest_system;
use super::update_target_system;
use super::Ai;
use super::AiTarget;
use crate::ability::AbilityId;
use crate::level::Floor;
use crate::movement::DesiredMove;
use crate::multiplayer::Action;
use crate::player::Abilities;
use crate::player::AbilityIds;
use crate::AbilityOffset;
use crate::Ally;
use crate::Enemy;
use crate::Faction;
use crate::To2d;
use crate::To3d;

#[derive(Component)]
pub struct ChargeAi {
    pub desired_range_squared: f32,
    /// The distance the target gets from the end of the path before we
    /// re-compute it.
    pub target_dist_squared: f32,
    pub intelligence: f32,
    /// Whether there is an obstruction between this entity and where it wants
    /// to shoot.
    pub gun_obstruction: bool,
    // pub move_loc: Option<Vec2>,
    pub ability_ids: AbilityIds,
}

impl Ai for ChargeAi {
    fn intelligence(&self) -> f32 {
        self.intelligence
    }
}

impl Default for ChargeAi {
    fn default() -> Self {
        let mut rng = rand::rng();
        let desired_range = rng.random_range(0.0..=4.0);
        Self {
            desired_range_squared: desired_range * desired_range,
            target_dist_squared: 3.0 * 3.0,
            intelligence: 1.0,
            gun_obstruction: true,
            ability_ids: AbilityIds {
                left_arm: AbilityId::from("gun"),
                ..Default::default()
            },
        }
    }
}

pub fn system_set() -> SystemConfigs {
    (
        target_closest_system::<Enemy, ChargeAi>,
        target_closest_system::<Ally, ChargeAi>,
        update_target_system::<Enemy, ChargeAi>,
        update_target_system::<Ally, ChargeAi>,
        check_obstructions::<Enemy>,
        check_obstructions::<Ally>,
        gun_system,
        move_system::<Enemy>,
        move_system::<Ally>,
    )
        .chain()
}

fn check_obstructions<T: Faction>(
    rapier_context: ReadDefaultRapierContext,
    mut ai_q: Query<(Entity, &AiTarget, &Transform, &AbilityOffset, &mut ChargeAi), With<T>>,
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

    for (entity, target, transform, ability_offset, mut ai) in &mut ai_q {
        let Some(target_entity) = target.entity else {
            ai.gun_obstruction = true;
            continue;
        };
        let pred = |e| exclude_walls_friends(entity, target_entity, e);
        let filter = QueryFilter::new().predicate(&pred);
        let origin = transform.translation + ability_offset.to_vec();

        let dir = target.loc.0 - transform.translation.to_2d();

        let ray = rapier_context.cast_ray(origin, dir.to_3d(0.0), 1.0, true, filter);

        ai.gun_obstruction = ray.is_some();
    }
}

fn gun_system(mut commands: Commands, mut ai_q: Query<(Entity, &ChargeAi, &Abilities)>) {
    for (entity, ai, abilities) in ai_q.iter_mut() {
        if !ai.gun_obstruction {
            // We'll just try to fire all abilities here, as dumb as that is.
            // Because shooting is first, and cooldowns exist, we'll shoot until
            // out of ammo, then reload, then repeat, so it's not tooo bad in
            // practice.
            Action::all_flags().fire_abilities(&mut commands, entity, abilities);
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct MoveQuery {
    entity: Entity,
    target: &'static mut AiTarget,
    transform: &'static Transform,
    ai: &'static ChargeAi,
    has_path: &'static mut HasPath,
    desired_move: &'static mut DesiredMove,
}

fn move_system<T: Faction>(
    mut ai_q: Query<MoveQuery, With<T>>,
    target_q: Query<&Transform, (With<T::Foe>, Without<T>)>,
    mut events: EventWriter<PathfindEvent>,
) {
    for mut ai in &mut ai_q {
        let Some(target_transform) = ai.target.entity.and_then(|e| target_q.get(e).ok()) else {
            // Can't do much without a target.
            continue;
        };
        let loc = ai.transform.translation.to_2d();
        let target_loc = target_transform.translation.to_2d();

        enum Task {
            Pathfind,
            Move,
            Stop,
        }

        let task = if let Some(final_dest) = ai.has_path.path.last() {
            // We still have a valid path.
            if final_dest.to_2d().distance_squared(target_loc) > ai.ai.target_dist_squared {
                // Target has moved too far; recompute path.
                Task::Pathfind
            } else if loc.distance_squared(target_loc) < ai.ai.desired_range_squared
                && !ai.ai.gun_obstruction
            {
                Task::Stop
            } else {
                Task::Move
            }
        } else {
            // We don't have a destination. We should always have one just in
            // case.
            Task::Pathfind
        };

        match task {
            Task::Pathfind => {
                events.send(PathfindEvent {
                    entity: ai.entity,
                    target: target_loc,
                });
            }
            Task::Move => {
                set_move(ai.has_path, ai.transform, ai.desired_move);
            }
            Task::Stop => {
                // TODO: Do something more interesting than stop when we get
                // close.
                ai.desired_move.dir = Vec2::ZERO;
            }
        }
    }
}
