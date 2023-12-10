use bevy_ecs::{
    bundle::Bundle, component::Component, entity::Entity, event::EventReader, system::Query,
};
use bevy_rapier3d::prelude::{ActiveEvents, CollisionEvent};
use smallvec::SmallVec;

#[derive(Bundle)]
pub struct TrackCollisionBundle {
    active_events: ActiveEvents,
    track_collisions: TrackCollisions,
}

impl TrackCollisionBundle {
    pub fn on() -> Self {
        Self {
            active_events: ActiveEvents::COLLISION_EVENTS,
            track_collisions: TrackCollisions::default(),
        }
    }

    pub fn off() -> Self {
        Self {
            active_events: ActiveEvents::empty(),
            track_collisions: TrackCollisions::default(),
        }
    }
}

/// Any entity with this component and `ActiveEvents::COLLISION_EVENTS` will be
/// updated every frame with its collision targets. It will also need to be a
/// `RigidBody::Dynamic`.
#[derive(Debug, Component, Default)]
pub struct TrackCollisions {
    pub targets: SmallVec<Entity, 4>,
}

impl TrackCollisions {
    pub fn remove(&mut self, target: Entity) {
        if let Some(idx) = self.targets.iter().position(|&entity| entity == target) {
            self.targets.swap_remove(idx);
        }
    }
}

fn remove_collision(
    entity: Entity,
    target: Entity,
    collisions_q: &mut Query<&mut TrackCollisions>,
) {
    if let Ok(mut colliding) = collisions_q.get_mut(entity) {
        colliding.remove(target);
    }
}

fn add_collision(entity: Entity, target: Entity, collisions_q: &mut Query<&mut TrackCollisions>) {
    if let Ok(mut colliding) = collisions_q.get_mut(entity) {
        colliding.targets.push(target);
    }
}

pub fn collision_system(
    mut collision_events: EventReader<CollisionEvent>,
    mut collisions_q: Query<&mut TrackCollisions>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                add_collision(*e1, *e2, &mut collisions_q);
                add_collision(*e2, *e1, &mut collisions_q);
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                remove_collision(*e1, *e2, &mut collisions_q);
                remove_collision(*e2, *e1, &mut collisions_q);
            }
        };
    }
}
