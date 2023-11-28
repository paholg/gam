use bevy_ecs::{
    component::Component,
    system::{Commands, Query},
};

use bevy_rapier3d::prelude::{ActiveEvents, Collider, Sensor};
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
    explosion_q: Query<(&Explosion, &Colliding)>,
    mut target_q: Query<&mut Health>,
) {
    for (explosion, colliding) in &explosion_q {
        for &target in &colliding.targets {
            if let Ok(mut health) = target_q.get_mut(target) {
                health.take(explosion.damage);
            }
        }
    }
}
