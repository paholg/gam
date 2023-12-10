use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{Added, With, Without},
    system::{Commands, Query, Res},
};
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{
    ActiveEvents, Ccd, Collider, ColliderMassProperties, ExternalForce, LockedAxes,
    ReadMassProperties, RigidBody, Sensor, Velocity,
};
use bevy_transform::components::{GlobalTransform, Transform};

use crate::{
    collision::TrackCollisions,
    level::InLevel,
    status_effect::StatusBundle,
    time::{Frame, FrameCounter},
    Health, Kind, Object, Shootable,
};

pub struct BulletSpawner {
    pub velocity: Vec3,
    pub position: Vec3,
    pub radius: f32,
    pub density: f32,
    pub bullet: Bullet,
    pub health: Health,
}

#[derive(Component)]
pub struct Bullet {
    pub shooter: Entity,
    pub expires_at: Frame,
    pub damage: f32,
}

impl BulletSpawner {
    pub fn spawn(self, commands: &mut Commands) {
        commands.spawn((
            Object {
                transform: Transform::from_translation(self.position).with_scale(Vec3::new(
                    self.radius,
                    self.radius,
                    self.radius,
                )),
                global_transform: GlobalTransform::default(),
                collider: Collider::ball(self.radius),
                foot_offset: (-self.radius).into(),
                mass_props: ColliderMassProperties::Density(self.density),
                body: RigidBody::Dynamic,
                force: ExternalForce::default(),
                velocity: Velocity {
                    linvel: self.velocity,
                    angvel: Vec3::ZERO,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
                mass: ReadMassProperties::default(),
                kind: Kind::Bullet,
                in_level: InLevel,
                statuses: StatusBundle::default(),
            },
            ActiveEvents::COLLISION_EVENTS,
            TrackCollisions::default(),
            Sensor,
            Ccd::enabled(),
            self.health,
            self.bullet,
        ));
    }
}

// FIXME: Currently, this registers bullet mass as 0.
pub fn kickback_system(
    bullet_q: Query<(&Velocity, &ReadMassProperties, &Bullet), Added<Bullet>>,
    mut shooter_q: Query<(&mut Velocity, &ReadMassProperties), Without<Bullet>>,
) {
    for (v, m, bullet) in bullet_q.iter() {
        let Ok((mut shooter_v, shooter_m)) = shooter_q.get_mut(bullet.shooter) else {
            continue;
        };
        shooter_v.linvel -= v.linvel * m.mass / shooter_m.mass;
    }
}

pub fn despawn_system(
    mut commands: Commands,
    tick_counter: Res<FrameCounter>,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, shot) in query.iter_mut() {
        if shot.expires_at.before_now(&tick_counter) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn collision_system(
    mut bullet_q: Query<(
        &mut Health,
        &Bullet,
        &ReadMassProperties,
        &Velocity,
        &TrackCollisions,
    )>,
    mut health_q: Query<&mut Health, Without<Bullet>>,
    mut momentum_q: Query<(&mut Velocity, &ReadMassProperties), Without<Bullet>>,
    shootable_q: Query<(), With<Shootable>>,
) {
    for (mut health, bullet, bullet_mass, bullet_velocity, colliding) in &mut bullet_q {
        let mut should_die = false;
        for &target in &colliding.targets {
            if shootable_q.get(target).is_ok() {
                should_die = true;
            }
            if let Ok(mut health) = health_q.get_mut(target) {
                health.take(bullet.damage);
            }

            if let Ok((mut velocity, mass)) = momentum_q.get_mut(target) {
                // TODO: Add angvel maybe?
                velocity.linvel =
                    bullet_mass.mass * bullet_velocity.linvel / mass.mass + velocity.linvel;
            }
        }

        if should_die {
            health.die();
        }
    }
}
