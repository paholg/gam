use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    query::{With, Without, WorldQuery},
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Commands, Query, Res},
};
use bevy_math::Vec2;
use bevy_rapier3d::prelude::{QueryFilter, RapierContext, Velocity};
use bevy_transform::components::Transform;
use rand::{thread_rng, Rng};

use crate::{
    ability::{properties::AbilityProps, Ability},
    level::Floor,
    movement::DesiredMove,
    status_effect::StatusEffects,
    time::TickCounter,
    AbilityOffset, Ally, Cooldowns, Enemy, Energy, Faction, Target, To2d, To3d,
};

use super::{
    pathfind::{set_move, HasPath, PathfindEvent},
    target_closest_system, update_target_system, Ai, AiTarget,
};

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
}

impl Ai for ChargeAi {
    fn intelligence(&self) -> f32 {
        self.intelligence
    }
}

impl Default for ChargeAi {
    fn default() -> Self {
        let mut rng = thread_rng();
        let desired_range = rng.gen_range(0.0..=6.0);
        Self {
            desired_range_squared: desired_range * desired_range,
            target_dist_squared: 3.0 * 3.0,
            intelligence: 1.0,
            gun_obstruction: true,
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
    rapier_context: Res<RapierContext>,
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

fn gun_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut ai_q: Query<(
        Entity,
        &mut Cooldowns,
        &mut Energy,
        &Velocity,
        &Transform,
        &mut StatusEffects,
        &AbilityOffset,
        &ChargeAi,
    )>,
    props: Res<AbilityProps>,
) {
    for (
        entity,
        mut cooldowns,
        mut energy,
        velocity,
        transform,
        mut status_effects,
        ability_offset,
        ai,
    ) in ai_q.iter_mut()
    {
        if !ai.gun_obstruction {
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

#[derive(WorldQuery)]
#[world_query(mutable)]
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
            } else if loc.distance_squared(target_loc) < ai.ai.desired_range_squared {
                Task::Stop
            } else {
                Task::Move
            }
        } else {
            // We don't have a destination.
            if loc.distance_squared(target_loc) < ai.ai.desired_range_squared {
                Task::Stop
            } else {
                Task::Pathfind
            }
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
