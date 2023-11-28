use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventReader,
    query::With,
    system::{Commands, Query},
};
use bevy_rapier3d::prelude::CollisionEvent;
use bevy_utils::HashMap;
use smallvec::SmallVec;

#[derive(Debug, Component)]
pub struct TrackCollisions;

#[derive(Debug, Component, Default)]
pub struct Colliding {
    pub targets: SmallVec<[Entity; 4]>,
}

pub fn clear_colliding_system(
    mut commands: Commands,
    collisions_q: Query<Entity, With<Colliding>>,
) {
    for entity in collisions_q.iter() {
        commands.entity(entity).remove::<Colliding>();
    }
}

pub fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<Entity, With<TrackCollisions>>,
) {
    let mut collisions = HashMap::new();

    let mut push_collision = |entity, target| {
        collisions
            .entry(entity)
            .or_insert(Colliding::default())
            .targets
            .push(target)
    };

    for event in collision_events.read() {
        // We only care about the beginning of collisions, for now.
        let CollisionEvent::Started(e1, e2, _flags) = event else {
            continue;
        };
        let e1 = *e1;
        let e2 = *e2;

        let mut tracked = false;

        if let Ok(entity) = query.get_mut(e1) {
            push_collision(entity, e2);
            tracked = true;
        }

        if let Ok(entity) = query.get_mut(e2) {
            push_collision(entity, e1);
            tracked = true;
        }

        debug_assert!(tracked);
        if !tracked {
            tracing::warn!("Detected CollisionEvent, but no colliders with TrackCollisions");
        }
    }

    for (entity, colliding) in collisions {
        commands.entity(entity).insert(colliding);
    }
}
