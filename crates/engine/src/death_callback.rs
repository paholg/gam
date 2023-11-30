use bevy_ecs::{
    component::Component,
    system::{Commands, Query, Res},
};

use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, QueryFilter, QueryFilterFlags, RapierContext, Sensor,
};
use bevy_transform::components::Transform;

use crate::{
    collision::{Colliding, TrackCollisions},
    time::Tick,
    Health, Object,
};

#[derive(Debug, Component)]
pub enum DeathCallback {
    Explosion(ExplosionCallback),
}

impl DeathCallback {
    pub fn call(&self, commands: &mut Commands, transform: &Transform) {
        match self {
            DeathCallback::Explosion(explosion) => explosion.call(commands, transform),
        }
    }
}

#[derive(Debug)]
pub struct ExplosionCallback {
    pub damage: f32,
    pub radius: f32,
}

#[derive(Debug, Component)]
pub struct Explosion {
    pub damage: f32,
}

impl ExplosionCallback {
    pub fn call(&self, commands: &mut Commands, transform: &Transform) {
        commands.spawn((
            Object {
                transform: *transform,
                collider: Collider::ball(self.radius),
                foot_offset: (-self.radius).into(),
                body: bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
                ..Default::default()
            },
            Sensor,
            Explosion {
                damage: self.damage,
            },
            ActiveEvents::COLLISION_EVENTS,
            TrackCollisions,
            // Zero health with 1 tick delay ensures an explosion lives for
            // exactly 1 frame.
            Health::new_with_delay(0.0, Tick(1)),
        ));
    }
}

pub fn explosion_collision_system(
    rapier_context: Res<RapierContext>,
    explosion_q: Query<(&Explosion, &Transform, &Colliding)>,
    mut target_q: Query<(&Transform, &mut Health)>,
) {
    let query_filter = QueryFilter {
        flags: QueryFilterFlags::ONLY_FIXED,
        ..Default::default()
    };
    for (explosion, transform, colliding) in &explosion_q {
        for &target in &colliding.targets {
            if let Ok((target_transform, mut health)) = target_q.get_mut(target) {
                let origin = transform.translation;
                let dir = target_transform.translation - origin;
                let wall_collision =
                    rapier_context.cast_ray(origin, dir, f32::MAX, true, query_filter);
                if let Some((_entity, toi)) = wall_collision {
                    let delta_wall = dir * toi;
                    if delta_wall.length_squared() < dir.length_squared() {
                        // There is a wall between us and the target!
                        // TODO: We're just checking between the center of the
                        // exploder and the target; we're going to miss some
                        // explosions that should hit.
                        continue;
                    }
                }
                health.take(explosion.damage);
            }
        }
    }
}
