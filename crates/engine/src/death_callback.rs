use bevy_ecs::{
    component::Component,
    system::{Commands, Query, Res},
};

use bevy_rapier3d::prelude::{
    ActiveEvents, Collider, QueryFilter, QueryFilterFlags, RapierContext, Sensor,
};
use bevy_transform::components::Transform;

use crate::{
    ability::properties::ExplosionProps,
    collision::{Colliding, TrackCollisions},
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
    pub props: ExplosionProps,
}

#[derive(Debug, Copy, Clone)]
pub enum ExplosionKind {
    FragGrenade,
    HealGrenade,
    SeekerRocket,
}

#[derive(Debug, Component)]
pub struct Explosion {
    pub damage: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub growth_rate: f32,
    pub kind: ExplosionKind,
}

impl ExplosionCallback {
    pub fn call(&self, commands: &mut Commands, transform: &Transform) {
        commands.spawn((
            Object {
                transform: *transform,
                collider: Collider::ball(self.props.min_radius),
                // Foot offset doesn't really make sense for an explosion, I think.
                foot_offset: 0.0.into(),
                body: bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
                ..Default::default()
            },
            Sensor,
            Explosion {
                damage: self.props.damage,
                min_radius: self.props.min_radius,
                max_radius: self.props.max_radius,
                growth_rate: (self.props.max_radius - self.props.min_radius)
                    / self.props.duration.0 as f32,
                kind: self.props.kind,
            },
            ActiveEvents::COLLISION_EVENTS,
            TrackCollisions,
            Health::new_with_delay(0.0, self.props.duration),
        ));
    }
}

pub fn explosion_grow_system(mut explosion_q: Query<(&Explosion, &mut Collider)>) {
    for (explosion, mut collider) in &mut explosion_q {
        let mut ball = collider.as_ball_mut().unwrap();
        let new_radius = ball.radius() + explosion.growth_rate;
        ball.set_radius(new_radius);
    }
}

pub fn explosion_collision_system(
    rapier_context: Res<RapierContext>,
    explosion_q: Query<(&Explosion, &Transform, &Colliding)>,
    mut target_q: Query<(&Transform, &mut Health)>,
) {
    let wall_filter = QueryFilter {
        flags: QueryFilterFlags::ONLY_FIXED,
        ..Default::default()
    };
    for (explosion, transform, colliding) in &explosion_q {
        for &target in &colliding.targets {
            if let Ok((target_transform, mut health)) = target_q.get_mut(target) {
                let origin = transform.translation;
                let dir = target_transform.translation - origin;
                let wall_collision =
                    rapier_context.cast_ray(origin, dir, f32::MAX, true, wall_filter);
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
