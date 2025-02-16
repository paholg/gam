use bevy_app::Plugin;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::system::Commands;
use bevy_ecs::system::In;
use bevy_ecs::system::Query;
use bevy_ecs::system::Resource;
use bevy_ecs::system::SystemId;
use bevy_math::Vec3;
use bevy_rapier3d::plugin::ReadDefaultRapierContext;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::QueryFilter;
use bevy_rapier3d::prelude::QueryFilterFlags;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Sensor;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use crate::collision::TrackCollisionBundle;
use crate::collision::TrackCollisions;
use crate::level::InLevel;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::Dur;
use crate::GameSet;
use crate::Health;
use crate::MassBundle;
use crate::Object;
use crate::To2d;
use crate::To3d;
use crate::SCHEDULE;

pub struct ExplosionPlugin;
impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let callback = ExplosionCallback {
            system: app.register_system(explosion_callback),
        };
        app.insert_resource(callback).add_systems(
            SCHEDULE,
            (explosion_collision_system, explosion_grow_system).in_set(GameSet::Stuff),
        );
    }
}

#[derive(Resource)]
pub struct ExplosionCallback {
    pub system: SystemId<In<Entity>>,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct ExplosionProps {
    pub damage: f32,
    pub force: f32,
    pub min_radius: f32,
    pub max_radius: f32,
    pub duration: Dur,
    pub kind: ExplosionKind,
}

// TODO: Get rid of this enum.
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

impl From<&ExplosionProps> for Explosion {
    fn from(props: &ExplosionProps) -> Self {
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

fn explosion_callback(
    In(entity): In<Entity>,
    mut commands: Commands,
    query: Query<(&Transform, &ExplosionProps)>,
) {
    let Ok((transform, props)) = query.get(entity) else {
        return;
    };
    let mut transform = *transform;
    transform.scale = Vec3::splat(props.min_radius);
    commands.spawn((
        // TODO: This should not be an Object, a lot of these things don't
        // make sense.
        Object {
            transform: (transform),
            collider: Collider::ball(1.0),
            // Foot offset doesn't really make sense for an explosion, I think.
            foot_offset: 0.0.into(),
            body: RigidBody::KinematicPositionBased,
            mass: MassBundle::new(0.0),
            velocity: Velocity::zero(),
            force: ExternalForce::default(),
            locked_axes: LockedAxes::empty(),
            in_level: InLevel,
            statuses: StatusProps {
                thermal_mass: 1.0,
                capacitance: 1.0,
            }
            .into(),
            collisions: TrackCollisionBundle::on(),
        },
        Explosion::from(props),
        Sensor,
        Health::new_with_delay(0.0, props.duration),
    ));
}

fn explosion_grow_system(mut explosion_q: Query<(&Explosion, &mut Transform, &TimeDilation)>) {
    for (explosion, mut transform, time_dilation) in &mut explosion_q {
        // Assume all scale axes are the same.
        let radius = transform.scale.x;
        let new_radius = radius + explosion.growth_rate * time_dilation.factor();
        transform.scale = Vec3::splat(new_radius);
    }
}

fn explosion_collision_system(
    rapier_context: ReadDefaultRapierContext,
    explosion_q: Query<(&Explosion, &Transform, &TrackCollisions, &TimeDilation)>,
    mut target_q: Query<(&Transform, &mut Health, &mut ExternalForce, &TimeDilation)>,
) {
    let wall_filter = QueryFilter {
        flags: QueryFilterFlags::ONLY_FIXED,
        ..Default::default()
    };
    for (explosion, transform, colliding, dilation) in &explosion_q {
        // Dilated explosions have their lifetimes and grow rates affected, so
        // their damage should be too. This way, a full explosion always does a
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
