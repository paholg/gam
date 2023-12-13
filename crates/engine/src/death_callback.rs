use bevy_ecs::{
    component::Component,
    system::{Commands, Query, Res},
};

use bevy_rapier3d::prelude::{
    Collider, ExternalForce, LockedAxes, QueryFilter, QueryFilterFlags, RapierContext, Sensor,
    Velocity,
};
use bevy_transform::components::Transform;

use crate::{
    ability::properties::ExplosionProps,
    collision::{TrackCollisionBundle, TrackCollisions},
    level::InLevel,
    status_effect::{StatusProps, TimeDilation},
    Health, Kind, MassBundle, Object, To2d, To3d,
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
    pub force: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub growth_rate: f32,
    pub kind: ExplosionKind,
}

impl From<ExplosionProps> for Explosion {
    fn from(props: ExplosionProps) -> Self {
        Self {
            damage: props.damage,
            force: props.force,
            min_radius: props.min_radius,
            max_radius: props.max_radius,
            growth_rate: (props.max_radius - props.min_radius) / props.duration,
            kind: props.kind,
        }
    }
}

impl ExplosionCallback {
    pub fn call(&self, commands: &mut Commands, transform: &Transform) {
        commands.spawn((
            // TODO: This should not be an Object, a lot of these things don't
            // make sense.
            Object {
                transform: (*transform).into(),
                collider: Collider::ball(self.props.min_radius),
                // Foot offset doesn't really make sense for an explosion, I think.
                foot_offset: 0.0.into(),
                body: bevy_rapier3d::prelude::RigidBody::KinematicPositionBased,
                mass: MassBundle::new(0.0),
                velocity: Velocity::zero(),
                force: ExternalForce::default(),
                locked_axes: LockedAxes::empty(),
                kind: Kind::Other,
                in_level: InLevel,
                statuses: StatusProps {
                    thermal_mass: 1.0,
                    capacitance: 1.0,
                }
                .into(),
                collisions: TrackCollisionBundle::on(),
            },
            Explosion::from(self.props),
            Sensor,
            Health::new_with_delay(0.0, self.props.duration),
        ));
    }
}

pub fn explosion_grow_system(mut explosion_q: Query<(&Explosion, &mut Collider, &TimeDilation)>) {
    for (explosion, mut collider, time_dilation) in &mut explosion_q {
        let mut ball = collider.as_ball_mut().unwrap();
        let new_radius = ball.radius() + explosion.growth_rate * time_dilation.factor();
        ball.set_radius(new_radius);
    }
}

pub fn explosion_collision_system(
    rapier_context: Res<RapierContext>,
    explosion_q: Query<(&Explosion, &Transform, &TrackCollisions, &TimeDilation)>,
    mut target_q: Query<(&Transform, &mut Health, &mut ExternalForce, &TimeDilation)>,
) {
    let wall_filter = QueryFilter {
        flags: QueryFilterFlags::ONLY_FIXED,
        ..Default::default()
    };
    for (explosion, transform, colliding, dilation) in &explosion_q {
        // Dilated explosions have their lifetimes and grow rates affected, so
        // their damage should be to. This way, a full explosion always does a
        // constant damage.
        let explosion_damage = explosion.damage * dilation.factor();
        let explosion_force = explosion.force * dilation.factor();
        for &target in &colliding.targets {
            if let Ok((target_transform, mut health, mut force, target_dilation)) =
                target_q.get_mut(target)
            {
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
                health.take(explosion_damage, target_dilation);
                let dir = (target_transform.translation.to_2d() - transform.translation.to_2d())
                    .normalize_or_zero()
                    .to_3d(0.0);
                force.force += dir * explosion_force;
            }
        }
    }
}
