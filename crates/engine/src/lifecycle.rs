use bevy_ecs::{
    entity::Entity,
    event::{Event, EventWriter},
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{Friction, LockedAxes, RapierContext, RigidBody};
use bevy_transform::components::Transform;

use crate::{
    ability::{Abilities, Ability, ABILITY_Y},
    ai::simple::Attitude,
    death_callback::DeathCallback,
    level::LevelProps,
    player::{character_collider, PlayerInfo},
    time::{Tick, TickCounter},
    Ai, Ally, Character, Cooldowns, Enemy, Energy, FootOffset, Health, Kind, NumAi, Object, Player,
    Shootable, PLAYER_HEIGHT, PLAYER_R,
};

pub const DEATH_Y: f32 = -2.0;

#[derive(Debug, Event)]
pub struct DeathEvent {
    pub transform: Transform,
    pub kind: Kind,
}

pub fn fall(mut query: Query<(&mut Health, &Transform, &FootOffset)>) {
    for (mut health, transform, foot_offset) in &mut query {
        if transform.translation.y + foot_offset.y < DEATH_Y {
            health.die();
        }
    }
}

// TODO: Use `Option<&DeathCallback>` instead of two queries.
pub fn die(
    mut commands: Commands,
    mut without_callback_q: Query<(Entity, &mut Health, &Transform, &Kind), Without<DeathCallback>>,
    mut with_callback_q: Query<(Entity, &mut Health, &Transform, &Kind, &DeathCallback)>,
    mut event_writer: EventWriter<DeathEvent>,
    tick_counter: Res<TickCounter>,
) {
    let events = without_callback_q
        .iter_mut()
        .filter_map(|(entity, mut health, &transform, &kind)| {
            if health.cur <= 0.0 {
                if health.death_delay == Tick(0) {
                    tracing::debug!(tick = ?tick_counter.tick, ?entity, ?health, ?transform, "DEATH");
                    commands.entity(entity).despawn_recursive();
                    Some(DeathEvent { transform, kind })
                } else {
                    health.death_delay -= Tick(1);
                    None
                }
            } else {
                None
            }
        });
    event_writer.send_batch(events);

    let more_events =
        with_callback_q
            .iter_mut()
            .filter_map(|(entity, mut health, &transform, &kind, callback)| {
                if health.cur <= 0.0 {
                    if health.death_delay == Tick(0) {
                        tracing::debug!(tick = ?tick_counter.tick, ?entity, ?health, ?transform, "DEATH WITH CALLBACK");
                        callback.call(&mut commands, &transform);
                        commands.entity(entity).despawn_recursive();
                        Some(DeathEvent { transform, kind })
                    } else { health.death_delay -= Tick(1);
                        None
                    }
                } else {
                    None
                }
            });
    event_writer.send_batch(more_events);
}

pub const ENERGY_REGEN: f32 = 0.5;

fn spawn_enemies(
    commands: &mut Commands,
    num: usize,
    level: &LevelProps,
    rapier_context: &RapierContext,
) {
    for _ in 0..num {
        let loc = level.point_in_plane(rapier_context);
        let abilities = Abilities::new(vec![Ability::Gun]);

        let id = commands
            .spawn((
                Enemy,
                Ai,
                Character {
                    health: Health::new(10.0),
                    energy: Energy::new(5.0, 0.2),
                    object: Object {
                        transform: Transform::from_translation(
                            loc + Vec3::new(0.0, 0.5 * PLAYER_HEIGHT, 0.0),
                        ),
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Enemy,
                        ..Default::default()
                    },
                    max_speed: Default::default(),
                    impulse: Default::default(),
                    force: Default::default(),
                    friction: Friction::default(),
                    status_effects: Default::default(),
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(&abilities),
                    abilities,
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                },
                Attitude::rand(level, rapier_context),
            ))
            .id();
        tracing::debug!(?id, "Spawning enemy");
    }
}

fn spawn_allies(
    commands: &mut Commands,
    num: usize,
    level: &LevelProps,
    rapier_context: &RapierContext,
) {
    for _ in 0..num {
        let loc = level.point_in_plane(rapier_context);
        let abilities = Abilities::new(vec![Ability::Gun]);
        let id = commands
            .spawn((
                Ally,
                Ai,
                Character {
                    health: Health::new(100.0),
                    energy: Energy::new(100.0, ENERGY_REGEN),
                    object: Object {
                        transform: Transform::from_translation(
                            loc + Vec3::new(0.0, 0.5 * PLAYER_HEIGHT, 0.0),
                        ),
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Ally,
                        ..Default::default()
                    },
                    max_speed: Default::default(),
                    impulse: Default::default(),
                    force: Default::default(),
                    friction: Friction::default(),
                    status_effects: Default::default(),
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(&abilities),
                    abilities,
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                },
            ))
            .id();
        tracing::debug!(?id, "Spawning ally");
    }
}

#[allow(clippy::too_many_arguments)]
pub fn reset(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    ally_query: Query<Entity, With<Ally>>,
    player_query: Query<Entity, With<Player>>,
    player_info_query: Query<&PlayerInfo>,
    mut num_ai: ResMut<NumAi>,
    level: Res<LevelProps>,
    rapier_context: Res<RapierContext>,
) {
    if enemy_query.iter().next().is_none() {
        num_ai.enemies += 1;
        spawn_enemies(&mut commands, num_ai.enemies, &level, &rapier_context);
    }

    if player_query.iter().next().is_none() {
        num_ai.enemies = num_ai.enemies.saturating_sub(1);
        for info in player_info_query.iter() {
            info.spawn_player(&mut commands);
        }
    }

    if ally_query.iter().next().is_none() {
        spawn_allies(&mut commands, num_ai.allies, &level, &rapier_context);
    }
}
