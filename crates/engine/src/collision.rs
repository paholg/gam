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
    pub targets: SmallVec<Entity, 4>,
}

impl Colliding {
    pub fn remove(&mut self, target: Entity) {
        if let Some(idx) = self.targets.iter().position(|&entity| entity == target) {
            self.targets.swap_remove(idx);
        }
    }
}

fn remove_collision(entity: Entity, target: Entity, collisions_q: &mut Query<&mut Colliding>) {
    if let Ok(mut colliding) = collisions_q.get_mut(entity) {
        colliding.remove(target);
    }
}

pub fn collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<Entity, With<TrackCollisions>>,
    mut collisions_q: Query<&mut Colliding>,
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
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
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
                    tracing::warn!(
                        "Detected CollisionEvent, but no colliders with TrackCollisions"
                    );
                }
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                remove_collision(*e1, *e2, &mut collisions_q);
                remove_collision(*e2, *e1, &mut collisions_q);
            }
        };
    }

    for (entity, colliding) in collisions {
        commands.entity(entity).insert(colliding);
    }
}
