use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::event::EventWriter;
use bevy_ecs::query::With;
use bevy_ecs::system::Commands;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::ResMut;
use bevy_hierarchy::DespawnRecursiveExt;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::CoefficientCombineRule;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::Friction;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::RapierContext;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Velocity;
use bevy_transform::components::Transform;

use crate::ability::Abilities;
use crate::ability::Ability;
use crate::ai::charge::ChargeAi;
use crate::ai::AiBundle;
use crate::collision::TrackCollisionBundle;
use crate::death_callback::DeathCallback;
use crate::level::InLevel;
use crate::level::LevelProps;
use crate::player::character_collider;
use crate::player::PlayerInfo;
use crate::status_effect::StatusProps;
use crate::status_effect::TimeDilation;
use crate::time::FrameCounter;
use crate::Ally;
use crate::Character;
use crate::CharacterMarker;
use crate::Cooldowns;
use crate::Enemy;
use crate::Energy;
use crate::FootOffset;
use crate::Health;
use crate::Kind;
use crate::MassBundle;
use crate::NumAi;
use crate::Object;
use crate::Player;
use crate::Shootable;
use crate::ABILITY_Y;
use crate::CONTACT_SKIN;
use crate::PLAYER_HEIGHT;
use crate::PLAYER_MASS;
use crate::PLAYER_R;

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
        &TimeDilation,
    )>,
    mut event_writer: EventWriter<DeathEvent>,
    tick_counter: Res<FrameCounter>,
) {
    let events = query.iter_mut().filter_map(
        |(entity, mut health, &transform, &kind, callback, time_dilation)| {
            if health.cur <= 0.0 && health.death_delay.tick(time_dilation) {
                tracing::debug!(tick = ?tick_counter.frame, ?entity, ?health, ?transform, "DEATH");
                if let Some(callback) = callback {
                    callback.call(&mut commands, &transform);
                }
                commands.entity(entity).despawn_recursive();
                Some(DeathEvent { transform, kind })
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
                        )
                        .into(),
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Enemy,
                        mass: MassBundle::new(PLAYER_MASS),
                        velocity: Velocity::zero(),
                        force: ExternalForce::default(),
                        in_level: InLevel,
                        statuses: StatusProps {
                            thermal_mass: 1.0,
                            capacitance: 1.0,
                        }
                        .into(),
                        collisions: TrackCollisionBundle::off(),
                    },
                    max_speed: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(),
                    abilities,
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                    marker: CharacterMarker,
                    contact_skin: CONTACT_SKIN,
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
                    object: Object {
                        transform: Transform::from_translation(
                            loc + Vec3::new(0.0, 0.5 * PLAYER_HEIGHT, 0.0),
                        )
                        .into(),
                        collider: character_collider(PLAYER_R, PLAYER_HEIGHT),
                        foot_offset: (-PLAYER_HEIGHT * 0.5).into(),
                        body: RigidBody::Dynamic,
                        locked_axes: LockedAxes::ROTATION_LOCKED,
                        kind: Kind::Ally,
                        mass: MassBundle::new(PLAYER_MASS),
                        velocity: Velocity::zero(),
                        force: ExternalForce::default(),
                        in_level: InLevel,
                        statuses: StatusProps {
                            thermal_mass: 1.0,
                            capacitance: 1.0,
                        }
                        .into(),
                        collisions: TrackCollisionBundle::off(),
                    },
                    contact_skin: CONTACT_SKIN,
                    health: Health::new(100.0),
                    energy: Energy::new(100.0, ENERGY_REGEN),
                    max_speed: Default::default(),
                    friction: Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                    },
                    shootable: Shootable,
                    cooldowns: Cooldowns::new(),
                    abilities,
                    desired_movement: Default::default(),
                    ability_offset: ((-PLAYER_HEIGHT * 0.5) + ABILITY_Y.y).into(),
                    marker: CharacterMarker,
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
