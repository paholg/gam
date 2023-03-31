use std::time::Duration;

use bevy::prelude::{
    Commands, Component, ComputedVisibility, Entity, GlobalTransform, Query, Res, Transform, Vec3,
    Visibility, With, Without,
};
use bevy_hanabi::ParticleEffect;
use bevy_rapier2d::prelude::{Collider, LockedAxes, RapierContext, RigidBody, Sensor, Velocity};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    asset_handler::{AssetHandler, HyperSprintEffect, ShotEffect},
    time::{Tick, TickCounter},
    Ally, Cooldowns, Enemy, Health, MaxSpeed, Object, PLAYER_R,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Shoot,
}

impl Ability {
    #[allow(clippy::too_many_arguments)]
    pub fn fire(
        &self,
        commands: &mut Commands,
        tick_counter: &TickCounter,
        #[cfg(feature = "graphics")] assets: &AssetHandler,
        entity: Entity,
        cooldowns: &mut Cooldowns,
        max_speed: &mut MaxSpeed,
        transform: &Transform,
        velocity: &Velocity,
    ) -> bool {
        match self {
            Ability::None => true,
            Ability::HyperSprint => {
                hyper_sprint(commands, tick_counter, entity, cooldowns, max_speed)
            }
            Ability::Shoot => shoot(
                commands,
                cooldowns,
                tick_counter,
                #[cfg(feature = "graphics")]
                assets,
                transform,
                velocity,
            ),
        }
    }
}

#[derive(Component)]
pub struct HyperSprinting {
    duration: Tick,
}

const HYPER_SPRINT_FACTOR: f32 = 7.0;
pub const HYPER_SPRINT_COOLDOWN: Tick = Tick::new(Duration::new(5, 0));
const HYPER_SPRINT_DURATION: Tick = Tick::new(Duration::from_secs_f32(0.15));

fn hyper_sprint(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    entity: Entity,
    cooldowns: &mut Cooldowns,
    max_speed: &mut MaxSpeed,
) -> bool {
    if cooldowns.hyper_sprint.before_now(tick_counter) {
        cooldowns.hyper_sprint = tick_counter.at(HYPER_SPRINT_COOLDOWN);
        max_speed.max_speed *= HYPER_SPRINT_FACTOR;
        max_speed.impulse *= HYPER_SPRINT_FACTOR;
        commands.entity(entity).insert(HyperSprinting {
            duration: tick_counter.at(HYPER_SPRINT_DURATION),
        });
        true
    } else {
        false
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<(Entity, &HyperSprinting, &mut MaxSpeed, &Transform)>,
    #[cfg(feature = "graphics")] mut effects: Query<
        (&mut ParticleEffect, &mut Transform),
        (With<HyperSprintEffect>, Without<HyperSprinting>),
    >,
    #[cfg(feature = "graphics")] assets: Res<AssetHandler>,
) {
    for (entity, hyper_sprinting, mut max_speed, sprinter_transform) in query.iter_mut() {
        #[cfg(feature = "graphics")]
        {
            let (mut effect, mut transform) =
                effects.get_mut(assets.hyper_sprint.effect_entity).unwrap();
            *transform = *sprinter_transform;
            effect.maybe_spawner().unwrap().reset();
        }
        if hyper_sprinting.duration.before_now(&tick_counter) {
            max_speed.max_speed /= HYPER_SPRINT_FACTOR;
            max_speed.impulse /= HYPER_SPRINT_FACTOR;
            commands.entity(entity).remove::<HyperSprinting>();
        }
    }
}

pub const SHOOT_COOLDOWN: Tick = Tick::new(Duration::from_millis(250));
const SHOT_DURATION: Tick = Tick::new(Duration::from_secs(10));
const SHOT_SPEED: f32 = 30.0;
pub const SHOT_R: f32 = 0.15;
const SHOT_DAMAGE: f32 = 20.0;

#[derive(Component)]
pub struct Shot {
    duration: Tick,
}

fn shoot(
    commands: &mut Commands,
    cooldowns: &mut Cooldowns,
    tick_counter: &TickCounter,
    #[cfg(feature = "graphics")] assets: &AssetHandler,
    transform: &Transform,
    velocity: &Velocity,
) -> bool {
    if cooldowns.shoot.before_now(tick_counter) {
        cooldowns.shoot = tick_counter.at(SHOOT_COOLDOWN);

        let dir = transform.rotation * Vec3::Y;
        let position = transform.translation + dir * (PLAYER_R + SHOT_R);
        let vel = dir.truncate() * SHOT_SPEED + velocity.linvel;
        commands.spawn((
            Object {
                #[cfg(feature = "graphics")]
                material: assets.shot.material.clone(),
                #[cfg(feature = "graphics")]
                mesh: assets.shot.mesh.clone(),
                transform: Transform::from_translation(position),
                global_transform: GlobalTransform::default(),
                #[cfg(feature = "graphics")]
                visibility: Visibility::Visible,
                #[cfg(feature = "graphics")]
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(SHOT_R),
                body: RigidBody::Dynamic,
                velocity: Velocity {
                    linvel: vel,
                    angvel: 0.0,
                },
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
            Sensor,
            Shot {
                duration: tick_counter.at(SHOT_DURATION),
            },
        ));
        true
    } else {
        false
    }
}

pub fn shot_despawn_system(
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<(Entity, &mut Shot)>,
) {
    for (entity, shot) in query.iter_mut() {
        if shot.duration.before_now(&tick_counter) {
            commands.entity(entity).despawn();
        }
    }
}

// Note: This iterates through all intersection_pairs. We should use one system
// for all such intersections to avoid duplicate work.
pub fn shot_hit_system(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    shot_query: Query<(Entity, &Transform), With<Shot>>,
    mut ally_query: Query<&mut Health, (With<Ally>, Without<Enemy>)>,
    mut enemy_query: Query<&mut Health, (With<Enemy>, Without<Ally>)>,
    miss_query: Query<Entity, Without<Health>>,
    #[cfg(feature = "graphics")] assets: Res<AssetHandler>,
    #[cfg(feature = "graphics")] mut effects: Query<
        (&mut ParticleEffect, &mut Transform),
        (With<ShotEffect>, Without<Shot>),
    >,
) {
    let mut shots_to_despawn: SmallVec<[(Entity, Transform); 10]> = smallvec::SmallVec::new();
    for (entity1, entity2, intersecting) in rapier_context.intersection_pairs() {
        if intersecting {
            let (shot_entity, shot_transform, target_entity) =
                if let Ok((e, t)) = shot_query.get(entity1) {
                    (e, t.to_owned(), entity2)
                } else if let Ok((e, t)) = shot_query.get(entity2) {
                    (e, t.to_owned(), entity1)
                } else {
                    continue;
                };

            if let Ok(mut health) = ally_query.get_mut(target_entity) {
                health.cur -= SHOT_DAMAGE;
                shots_to_despawn.push((shot_entity, shot_transform));
            }
            if let Ok(mut health) = enemy_query.get_mut(target_entity) {
                health.cur -= SHOT_DAMAGE;
                shots_to_despawn.push((shot_entity, shot_transform));
            }
            if miss_query.get(target_entity).is_ok() {
                shots_to_despawn.push((shot_entity, shot_transform));
            }
        }
    }
    shots_to_despawn.sort_by_key(|(entity, _transform)| *entity);
    shots_to_despawn.dedup_by_key(|(entity, _transform)| *entity);

    for (shot_entity, shot_transform) in shots_to_despawn.into_iter() {
        commands.entity(shot_entity).despawn();
        #[cfg(feature = "graphics")]
        {
            let (mut effect, mut transform) = effects.get_mut(assets.shot.effect_entity).unwrap();
            *transform = shot_transform;
            effect.maybe_spawner().unwrap().reset();
        }
    }
}
