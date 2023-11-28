use bevy_ecs::{
    entity::Entity,
    event::{Event, EventWriter},
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{Collider, Friction, LockedAxes, RigidBody};
use bevy_transform::components::{GlobalTransform, Transform};
use rand::Rng;

use crate::{
    ability::{Abilities, Ability},
    ai::simple::Attitude,
    death_callback::DeathCallback,
    player::PlayerInfo,
    time::{Tick, TickCounter},
    Ai, Ally, Character, Cooldowns, Enemy, Energy, Health, Kind, NumAi, Object, Player, Shootable,
    DAMPING, PLANE, PLAYER_R,
};

pub const DEATH_Z: f32 = -10.0;

#[derive(Debug, Event)]
pub struct DeathEvent {
    pub transform: Transform,
    pub kind: Kind,
}

pub fn fall(mut query: Query<(&mut Health, &Transform)>) {
    for (mut health, transform) in &mut query {
        if transform.translation.z < DEATH_Z {
            health.die();
        }
    }
}

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

pub fn point_in_plane() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen::<f32>() * (PLANE - PLAYER_R) - (PLANE - PLAYER_R) * 0.5;
    let y = rng.gen::<f32>() * (PLANE - PLAYER_R) - (PLANE - PLAYER_R) * 0.5;
    Vec3::new(x, y, 0.0)
}

fn spawn_enemies(commands: &mut Commands, num: usize) {
    for _ in 0..num {
        let loc = point_in_plane();
        let abilities = Abilities::new(vec![Ability::Gun]);

        let id = commands
            .spawn((
                Enemy,
                Ai,
                Character {
                    health: Health::new(10.0),
                    energy: Energy::new(5.0, 0.2),
                    object: Object {
                        transform: Transform::from_translation(loc),
                        global_transform: GlobalTransform::default(),
                        collider: Collider::capsule(
                            Vec3::new(0.0, 0.0, PLAYER_R),
                            Vec3::new(0.0, 0.0, 1.0 + PLAYER_R),
                            1.0,
                        ),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Enemy,
                        ..Default::default()
                    },
                    max_speed: Default::default(),
                    damping: DAMPING,
                    impulse: Default::default(),
                    force: Default::default(),
                    friction: Friction::default(),
                    status_effects: Default::default(),
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(&abilities),
                    abilities,
                },
                Attitude::rand(),
            ))
            .id();
        tracing::debug!(?id, "Spawning enemy");
    }
}

fn spawn_allies(commands: &mut Commands, num: usize) {
    for _ in 0..num {
        let loc = point_in_plane();
        let abilities = Abilities::new(vec![Ability::Gun]);
        let id = commands
            .spawn((
                Ally,
                Ai,
                Character {
                    health: Health::new(100.0),
                    energy: Energy::new(100.0, ENERGY_REGEN),
                    object: Object {
                        transform: Transform::from_translation(loc),
                        collider: Collider::capsule(
                            Vec3::new(0.0, 0.0, PLAYER_R),
                            Vec3::new(0.0, 0.0, 1.0 + PLAYER_R),
                            1.0,
                        ),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Ally,
                        ..Default::default()
                    },
                    max_speed: Default::default(),
                    damping: DAMPING,
                    impulse: Default::default(),
                    force: Default::default(),
                    friction: Friction::default(),
                    status_effects: Default::default(),
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(&abilities),
                    abilities,
                },
            ))
            .id();
        tracing::debug!(?id, "Spawning ally");
    }
}

pub fn reset(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    ally_query: Query<Entity, With<Ally>>,
    player_query: Query<Entity, With<Player>>,
    player_info_query: Query<&PlayerInfo>,
    mut num_ai: ResMut<NumAi>,
) {
    if enemy_query.iter().next().is_none() {
        num_ai.enemies += 1;
        spawn_enemies(&mut commands, num_ai.enemies);
    }

    if player_query.iter().next().is_none() {
        num_ai.enemies = num_ai.enemies.saturating_sub(1);
        for info in player_info_query.iter() {
            info.spawn_player(&mut commands);
        }
    }

    if ally_query.iter().next().is_none() {
        spawn_allies(&mut commands, num_ai.allies);
    }
}
