use bevy::prelude::{
    Commands, DespawnRecursiveExt, Entity, EventWriter, GlobalTransform, Query, ResMut, Transform,
    Vec3, With,
};
use bevy_rapier3d::prelude::{
    Collider, ExternalImpulse, LockedAxes, ReadMassProperties, RigidBody, Velocity,
};
use rand::Rng;

use crate::{
    ai::simple::Attitude,
    status_effect::StatusEffects,
    Ai, Ally, Character, Cooldowns, DeathEvent, Enemy, Energy, Health, MaxSpeed, NumAi, Player,
    DAMPING, PLANE, PLAYER_R,
};

pub fn die(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform)>,
    mut event_writer: EventWriter<DeathEvent>,
) {
    for (entity, health, &transform) in query.iter() {
        if health.cur <= 0.0 {
            event_writer.send(DeathEvent { transform });
            commands.entity(entity).despawn_recursive();
        }
    }
}

const ENERGY_REGEN: f32 = 0.3;

fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Player,
        Ally,
        Character {
            health: Health::new(100.0),
            energy: Energy::new(100.0, ENERGY_REGEN),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            collider: Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0), 1.0),
            body: RigidBody::Dynamic,
            max_speed: MaxSpeed::default(),
            velocity: Velocity::default(),
            damping: DAMPING,
            impulse: ExternalImpulse::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            mass: ReadMassProperties::default(),
            status_effects: StatusEffects::default(),
        },
        Cooldowns::default(),
    ));
}

pub fn point_in_plane() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen::<f32>() * (PLANE - PLAYER_R) - (PLANE - PLAYER_R) * 0.5;
    let y = rng.gen::<f32>() * (PLANE - PLAYER_R) - (PLANE - PLAYER_R) * 0.5;
    Vec3::new(x, y, 0.0)
}

fn spawn_enemies(commands: &mut Commands, num: usize) {
    for _ in 0..num {
        let loc = point_in_plane();
        commands.spawn((
            Enemy,
            Ai,
            Character {
                health: Health::new(10.0),
                energy: Energy::new(100.0, ENERGY_REGEN),
                transform: Transform::from_translation(loc),
                global_transform: GlobalTransform::default(),
                collider: Collider::capsule(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 2.0),
                    1.0,
                ),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed::default(),
                velocity: Velocity::default(),
                damping: DAMPING,
                impulse: ExternalImpulse::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                mass: ReadMassProperties::default(),
                status_effects: StatusEffects::default(),
            },
            Cooldowns::default(),
            Attitude::rand(),
        ));
    }
}

fn spawn_allies(commands: &mut Commands, num: usize) {
    for _ in 0..num {
        let loc = point_in_plane();
        commands.spawn((
            Ally,
            Ai,
            Character {
                health: Health::new(100.0),
                energy: Energy::new(100.0, ENERGY_REGEN),
                transform: Transform::from_translation(loc),
                global_transform: GlobalTransform::default(),
                collider: Collider::capsule(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 2.0),
                    1.0,
                ),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed::default(),
                velocity: Velocity::default(),
                damping: DAMPING,
                impulse: ExternalImpulse::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                mass: ReadMassProperties::default(),
                status_effects: StatusEffects::default(),
            },
            Cooldowns::default(),
        ));
    }
}

pub fn reset(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    ally_query: Query<Entity, With<Ally>>,
    player_query: Query<Entity, With<Player>>,
    mut num_ai: ResMut<NumAi>,
) {
    if enemy_query.iter().next().is_none() {
        num_ai.enemies += 1;
        spawn_enemies(&mut commands, num_ai.enemies);
    }

    #[cfg(not(feature = "train"))]
    {
        if player_query.iter().next().is_none() {
            num_ai.enemies = num_ai.enemies.saturating_sub(1);
            spawn_player(&mut commands);
        }
    }

    if ally_query.iter().next().is_none() {
        spawn_allies(&mut commands, num_ai.allies);
    }
}

pub fn energy_regen(mut query: Query<&mut Energy>) {
    for mut energy in &mut query {
        energy.cur += energy.regen;
        energy.cur = energy.cur.min(energy.max);
    }
}
