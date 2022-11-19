use std::time::Duration;

use bevy::{
    prelude::{
        shape::Icosphere, Assets, Color, Commands, Component, ComputedVisibility, Entity,
        GlobalTransform, Mesh, Query, Res, ResMut, StandardMaterial, Transform, Vec3, Visibility,
        With, Without,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_rapier3d::prelude::{Collider, LockedAxes, RapierContext, RigidBody, Sensor, Velocity};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{Health, MaxSpeed, Object, PlayerCooldowns, PLAYER_R};

/// Construct a cooldown timer
pub fn cooldown(cooldown: Duration) -> Timer {
    let mut timer = Timer::new(cooldown, TimerMode::Once);
    timer.tick(cooldown);
    timer
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Shoot,
}

impl Ability {
    pub fn fire(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        entity: Entity,
        cooldowns: &mut PlayerCooldowns,
        max_speed: &mut MaxSpeed,
        transform: &Transform,
        velocity: &Velocity,
    ) {
        match self {
            Ability::None => (),
            Ability::HyperSprint => hyper_sprint(commands, entity, cooldowns, max_speed),
            Ability::Shoot => shoot(commands, cooldowns, meshes, materials, transform, velocity),
        }
    }
}

#[derive(Component)]
pub struct HyperSprinting {
    duration: Timer,
}

const HYPER_SPRINT_FACTOR: f32 = 5.0;
pub const HYPER_SPRINT_COOLDOWN: Duration = Duration::new(5, 0);

fn hyper_sprint(
    commands: &mut Commands,
    entity: Entity,
    cooldowns: &mut PlayerCooldowns,
    max_speed: &mut MaxSpeed,
) {
    if cooldowns.hyper_sprint.finished() {
        cooldowns.hyper_sprint.reset();
        max_speed.0 *= HYPER_SPRINT_FACTOR;
        commands.entity(entity).insert(HyperSprinting {
            duration: Timer::from_seconds(0.15, TimerMode::Once),
        });
    } else {
        debug!(
            hyper_sprint = cooldowns.hyper_sprint.remaining_secs(),
            "Remaining"
        );
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HyperSprinting, &mut MaxSpeed)>,
    time: Res<Time>,
) {
    for (entity, mut hyper_sprinting, mut max_speed) in query.iter_mut() {
        if hyper_sprinting.duration.tick(time.delta()).just_finished() {
            max_speed.0 /= HYPER_SPRINT_FACTOR;
            commands.entity(entity).remove::<HyperSprinting>();
        }
    }
}

pub const SHOOT_COOLDOWN: Duration = Duration::from_millis(100);
const SHOT_DURATION: Duration = Duration::from_secs(2);
const SHOT_SPEED: f32 = 50.0;
const SHOT_R: f32 = 0.1;
const SHOT_DAMAGE: f32 = 21.0;

#[derive(Component)]
pub struct Shot {
    duration: Timer,
}

fn shoot(
    commands: &mut Commands,
    cooldowns: &mut PlayerCooldowns,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    transform: &Transform,
    velocity: &Velocity,
) {
    if cooldowns.shoot.finished() {
        cooldowns.shoot.reset();

        let dir = transform.rotation * Vec3::Y;
        let position = transform.translation + dir * (PLAYER_R + SHOT_R);
        let vel = velocity.linvel + dir * SHOT_SPEED;
        commands.spawn((
            Object {
                material: materials.add(Color::BLUE.into()),
                mesh: meshes.add(Mesh::from(Icosphere {
                    radius: SHOT_R,
                    subdivisions: 5,
                })),
                transform: Transform::from_translation(position),
                global_transform: GlobalTransform::default(),
                visibility: Visibility::VISIBLE,
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(SHOT_R),
                body: RigidBody::Dynamic,
                velocity: Velocity {
                    linvel: vel,
                    angvel: Vec3::ZERO,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
            Sensor,
            Shot {
                duration: Timer::new(SHOT_DURATION, TimerMode::Once),
            },
        ));
    } else {
        debug!(shoot = cooldowns.shoot.remaining_secs(), "Remaining");
    }
}

pub fn shot_despawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Shot)>,
) {
    for (entity, mut shot) in query.iter_mut() {
        if shot.duration.tick(time.delta()).finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn shot_hit_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<Entity, With<Shot>>,
    mut target_query: Query<(Entity, &mut Health)>,
) {
    for shot in shot_query.iter() {
        for (target, mut health) in target_query.iter_mut() {
            if rapier_context.intersection_pair(shot, target) == Some(true) {
                commands.entity(shot).despawn();
                health.cur -= SHOT_DAMAGE;
            }
        }
    }
}

pub fn shot_miss_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<Entity, With<Shot>>,
    mut target_query: Query<Entity, Without<Health>>,
) {
    for shot in shot_query.iter() {
        for target in target_query.iter_mut() {
            if rapier_context.intersection_pair(shot, target) == Some(true) {
                commands.entity(shot).despawn();
            }
        }
    }
}
