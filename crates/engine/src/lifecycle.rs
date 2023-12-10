use bevy_ecs::{
    entity::Entity,
    event::{Event, EventWriter},
    query::With,
    system::{Commands, Query, Res, ResMut},
};
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Friction, LockedAxes, RapierContext, RigidBody,
};
use bevy_transform::components::Transform;

use crate::{
    ability::{Abilities, Ability},
    ai::{charge::ChargeAi, AiBundle},
    death_callback::DeathCallback,
    level::LevelProps,
    player::{character_collider, PlayerInfo},
    time::FrameCounter,
    Ally, Character, Cooldowns, Enemy, Energy, FootOffset, Health, Kind, NumAi, Object, Player,
    Shootable, ABILITY_Y, PLAYER_HEIGHT, PLAYER_R,
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

pub fn die(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Health,
        &Transform,
        &Kind,
        Option<&DeathCallback>,
    )>,
    mut event_writer: EventWriter<DeathEvent>,
    tick_counter: Res<FrameCounter>,
) {
    let events = query.iter_mut().filter_map(
        |(entity, mut health, &transform, &kind, callback)| {
            if health.cur <= 0.0 {
                if health.death_delay.is_zero() {
                    tracing::debug!(tick = ?tick_counter.frame, ?entity, ?health, ?transform, "DEATH");
                    if let Some(callback) = callback {
                        callback.call(&mut commands, &transform);
                    }
                    commands.entity(entity).despawn_recursive();
                    Some(DeathEvent { transform, kind })
                } else {
                    health.death_delay.reduce_one();
                    None
                }
            } else {
                None
            }
        },
    );
    event_writer.send_batch(events);
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
                AiBundle::<ChargeAi>::default(),
                Character {
                    health: Health::new(10.0),
                    energy: Energy::new(20.0, 0.2),
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
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(&abilities),
                    abilities,
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                },
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
                AiBundle::<ChargeAi>::default(),
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
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
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
    mut player_query: Query<(Entity, &mut Health, &mut Energy, &mut Cooldowns), With<Player>>,
    player_info_query: Query<&PlayerInfo>,
    mut num_ai: ResMut<NumAi>,
    level: Res<LevelProps>,
    rapier_context: Res<RapierContext>,
) {
    if enemy_query.iter().next().is_none() {
        num_ai.enemies += 1;
        spawn_enemies(&mut commands, num_ai.enemies, &level, &rapier_context);

        for (_entity, mut health, mut energy, mut cooldowns) in &mut player_query {
            health.cur = health.max;
            energy.cur = energy.max;
            cooldowns.reset();
        }
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
