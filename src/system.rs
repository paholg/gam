use bevy::prelude::{
    Commands, DespawnRecursiveExt, Entity, EventWriter, GlobalTransform, Query, Res, ResMut,
    Transform, Vec2, Vec3, With,
};
use bevy_rapier3d::prelude::{
    Collider, ExternalImpulse, LockedAxes, ReadMassProperties, RigidBody, Velocity,
};
use rand::Rng;
use tracing::info;

use crate::{
    ai::simple::Attitude, status_effect::StatusEffects, time::TickCounter, Ai, Ally, Character,
    Cooldowns, DeathEvent, Enemy, Energy, Health, MaxSpeed, Player, Score, SpawnPeriod, DAMPING,
    PLANE, PLAYER_R,
};

pub fn die(
    mut commands: Commands,
    query: Query<(Entity, &Health, &Transform)>,
    mut event_writer: EventWriter<DeathEvent>,
    mut score: ResMut<Score>,
) {
    for (entity, health, &transform) in query.iter() {
        if health.cur <= 0.0 {
            event_writer.send(DeathEvent { transform });
            commands.entity(entity).despawn_recursive();
            score.0 += 1;
        }
    }
}

const ENERGY_REGEN: f32 = 0.5;

fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Player { target: Vec2::ZERO },
        Ally,
        Character {
            health: Health::new(100.0),
            energy: Energy::new(100.0, ENERGY_REGEN),
            global_transform: GlobalTransform::default(),
            collider: Collider::capsule(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0), 1.0),
            body: RigidBody::Dynamic,
            damping: DAMPING,
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            ..Default::default()
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
                energy: Energy::new(5.0, 0.2),
                transform: Transform::from_translation(loc),
                global_transform: GlobalTransform::default(),
                collider: Collider::capsule(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 2.0),
                    1.0,
                ),
                body: RigidBody::Dynamic,
                damping: DAMPING,
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                ..Default::default()
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
                collider: Collider::capsule(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 2.0),
                    1.0,
                ),
                body: RigidBody::Dynamic,
                damping: DAMPING,
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
                ..Default::default()
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
    mut score: ResMut<Score>,
    tick_counter: Res<TickCounter>,
    mut spawn_period: ResMut<SpawnPeriod>,
) {
    if player_query.iter().next().is_none() {
        spawn_player(&mut commands);
        score.0 = 0;
    }

    if spawn_period.next.before_now(&tick_counter) {
        info!(next = ?spawn_period.next, "Spawning");
        spawn_enemies(&mut commands, 1);
        spawn_period.decrease();
        spawn_period.next = tick_counter.at(spawn_period.period);
    }
}

pub fn energy_regen(mut query: Query<&mut Energy>) {
    for mut energy in &mut query {
        energy.cur += energy.regen;
        energy.cur = energy.cur.min(energy.max);
    }
}
