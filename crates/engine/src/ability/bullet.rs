use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Added;
use bevy_ecs::query::With;
use bevy_ecs::query::Without;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::Ccd;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::ReadMassProperties;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use crate::collision::TrackCollisionBundle;
use crate::collision::TrackCollisions;
use crate::level::InLevel;
use crate::lifecycle::Lifetime;
use crate::status_effect::StatusProps;
use crate::status_effect::Temperature;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::Health;
use crate::MassBundle;
use crate::Object;
use crate::Shootable;

#[derive(Debug, Copy, Clone)]
pub struct BulletProps {
    pub radius: f32,
    pub mass: f32,
    pub health: f32,
    pub lifetime: Dur,
    pub damage: f32,
    pub heat: f32,
}

pub struct BulletSpawner<G> {
    pub shooter: Entity,
    pub velocity: Vec3,
    pub position: Vec3,
    pub props: BulletProps,
    pub gun_kind: G,
}

#[derive(Component)]
pub struct Bullet {
    pub shooter: Entity,
    pub damage: f32,
    pub heat: f32,
}

impl<G: Component> BulletSpawner<G> {
    pub fn spawn(self, commands: &mut Commands) {
        commands.spawn((
            Object {
                transform: Transform::from_translation(self.position)
                    .with_scale(Vec3::splat(self.props.radius)),
                collider: Collider::ball(1.0),
                foot_offset: (-self.props.radius).into(),
                mass: MassBundle::new(self.props.mass),
                body: RigidBody::Dynamic,
                force: ExternalForce::default(),
                velocity: Velocity {
                    linvel: self.velocity,
                    angvel: Vec3::ZERO,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Y,
                in_level: InLevel,
                statuses: StatusProps {
                    thermal_mass: 1.0,
                    capacitance: 1.0,
                }
                .into(),
                collisions: TrackCollisionBundle::on(),
            },
            Lifetime::new(self.props.lifetime),
            Sensor,
            Ccd::enabled(),
            Health::new(self.props.health),
            Bullet {
                shooter: self.shooter,
                damage: self.props.damage,
                heat: self.props.heat,
            },
        ));
    }
}

pub fn kickback_system(
    bullet_q: Query<(&Velocity, &ReadMassProperties, &Bullet), Added<Bullet>>,
    mut shooter_q: Query<(&mut Velocity, &ReadMassProperties), Without<Bullet>>,
) {
    for (v, m, bullet) in bullet_q.iter() {
        let Ok((mut shooter_v, shooter_m)) = shooter_q.get_mut(bullet.shooter) else {
            continue;
        };
        shooter_v.linvel -= v.linvel * m.mass / shooter_m.mass;
        debug_assert!(m.mass > 0.0, "bullet spawned with 0 mass");
        debug_assert!(shooter_m.mass > 0.0, "bullet shooter has 0 mass");
        debug_assert!(
            !shooter_v.linvel.is_nan(),
            "NaN velocity after kickback. Bullet: v: {v:?}, m: {m:?}, shooter_m: {shooter_m:?}",
        );
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
    mut health_q: Query<(&mut Health, &mut Temperature, &TimeDilation), Without<Bullet>>,
    mut momentum_q: Query<(&mut Velocity, &ReadMassProperties), Without<Bullet>>,
    shootable_q: Query<(), With<Shootable>>,
) {
    for (mut health, bullet, bullet_mass, bullet_velocity, colliding) in &mut bullet_q {
        let mut should_die = false;
        for &target in &colliding.targets {
            if shootable_q.get(target).is_ok() {
                should_die = true;
            }
            if let Ok((mut health, mut temperature, dilation)) = health_q.get_mut(target) {
                health.take(bullet.damage, dilation);
                temperature.heat(bullet.heat);
            }

            if let Ok((mut velocity, mass)) = momentum_q.get_mut(target) {
                // TODO: This should never come up.
                if !mass.mass.is_normal() {
                    tracing::warn!("bullet hit something with no mass, mass: {mass:?}");
                    continue;
                }
                // TODO: Add angvel maybe?
                velocity.linvel =
                    bullet_mass.mass * bullet_velocity.linvel / mass.mass + velocity.linvel;
                debug_assert!(
                    !velocity.linvel.is_nan(),
                    "NaN velocity after bullet collision. Mass: {mass:?}, Bullet: \
                     {bullet_velocity:?}, bullet_mass: {bullet_mass:?}",
                );
            }
        }

        if should_die {
            health.die();
        }
    }
}
