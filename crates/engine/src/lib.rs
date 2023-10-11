#![feature(
    duration_consts_float,
    div_duration,
    const_fn_floating_point_arithmetic
)]
#![allow(clippy::type_complexity)]

pub mod ability;
pub mod ai;
pub mod physics;
pub mod player;
pub mod status_effect;
pub mod system;
pub mod time;

use std::{collections::HashMap, time::Duration};

use ability::{grenade::GrenadeLandEvent, properties::AbilityProps, Ability, ShotHitEvent};
use bevy_app::{App, FixedUpdate, Plugin, Startup};
use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    event::Event,
    schedule::{IntoSystemConfigs, State, States},
    system::{Commands, Res, Resource},
};
use bevy_math::{Quat, Vec2, Vec3};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalImpulse, LockedAxes, ReadMassProperties,
    RigidBody, Velocity,
};
use bevy_time::fixed_timestep::FixedTime;
use bevy_transform::components::{GlobalTransform, Transform};
use physics::PhysicsPlugin;
use status_effect::StatusEffects;
use time::{Tick, TickPlugin, TIMESTEP};

#[derive(States, PartialEq, Eq, Debug, Copy, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
    Paused,
}

#[derive(Event)]
pub struct DeathEvent {
    pub transform: Transform,
}

pub const PLAYER_R: f32 = 1.0;
const IMPULSE: f32 = 15.0;
const DAMPING: Damping = Damping {
    linear_damping: 5.0,
    angular_damping: 0.0,
};

pub const PLANE: f32 = 50.0;

#[derive(Component, Default)]
pub struct Health {
    pub cur: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { cur: max, max }
    }

    pub fn take(&mut self, dmg: f32) {
        // Note: Damage can be negative (for healing) so we need to clamp by
        // both min (0) and max.
        self.cur = (self.cur - dmg).clamp(0.0, self.max);
    }
}

#[derive(Component, Default)]
pub struct Energy {
    pub cur: f32,
    pub max: f32,
    pub regen: f32,
}

impl Energy {
    pub fn new(max: f32, regen: f32) -> Self {
        Self {
            cur: max,
            max,
            regen,
        }
    }

    pub fn try_use(&mut self, cost: f32) -> bool {
        if self.cur >= cost {
            self.cur -= cost;
            true
        } else {
            false
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct MaxSpeed {
    pub impulse: f32,
}

impl Default for MaxSpeed {
    fn default() -> Self {
        Self { impulse: IMPULSE }
    }
}

/// Indicate this entity is a player. Currently, we assume one player.
#[derive(Component)]
pub struct Player {
    pub target: Vec2,
}

/// Indicate this entity is controlled by AI.
#[derive(Component)]
pub struct Ai;

/// Indicate this entity is on the enemy team.
#[derive(Component)]
pub struct Enemy;

/// Indicate this entity is on the players' team.
#[derive(Component)]
pub struct Ally;

/// Indicates that this entity can be hit by shots; think characters and walls.
#[derive(Component, Default)]
pub struct Shootable;

#[derive(Component)]
pub struct Cooldowns {
    // TODO: Make a nohash hashmap
    map: HashMap<Ability, Tick>,
}

impl Cooldowns {
    pub fn with_abilities(abilities: impl IntoIterator<Item = Ability>) -> Self {
        let map = abilities
            .into_iter()
            .map(|ability| (ability, Tick::default()))
            .collect();
        Self { map }
    }
}

#[derive(Bundle, Default)]
pub struct Object {
    transform: Transform,
    global_transform: GlobalTransform,
    collider: Collider,
    mass_props: ColliderMassProperties,
    body: RigidBody,
    velocity: Velocity,
    locked_axes: LockedAxes,
    mass: ReadMassProperties,
}

#[derive(Bundle)]
struct Character {
    object: Object,
    health: Health,
    energy: Energy,
    max_speed: MaxSpeed,
    damping: Damping,
    impulse: ExternalImpulse,
    status_effects: StatusEffects,
    shootable: Shootable,
    cooldowns: Cooldowns,
}

#[derive(Resource)]
pub struct NumAi {
    pub enemies: usize,
    pub allies: usize,
}

/// This plugin contains everything needed to run the game headlessly.
pub struct GamPlugin;

impl Plugin for GamPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FixedTime::new(Duration::from_secs_f32(TIMESTEP)));

        app.add_state::<AppState>()
            .insert_resource(AbilityProps::default())
            .insert_resource(NumAi {
                enemies: 0,
                allies: 0,
            })
            .add_plugins(TickPlugin)
            .add_systems(Startup, setup)
            .add_engine_tick_systems((
                ability::hyper_sprint_system,
                ability::shot_despawn_system,
                ability::grenade::grenade_land_system,
            ))
            .add_event::<GrenadeLandEvent>()
            .add_engine_tick_systems((
                ability::grenade::grenade_explode_system,
                ability::grenade::explosion_despawn_system,
            ))
            .add_event::<ShotHitEvent>()
            .add_event::<DeathEvent>()
            .add_engine_tick_systems((ability::shot_hit_system, ability::shot_kickback_system))
            .add_plugins(ai::simple::SimpleAiPlugin)
            .add_engine_tick_systems((system::die, system::energy_regen, system::reset))
            .add_plugins(PhysicsPlugin);
    }
}

pub trait FixedTimestepSystem {
    fn add_engine_tick_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self;
}

pub fn game_running(state: Res<State<AppState>>) -> bool {
    state.get() == &AppState::Running
}

impl FixedTimestepSystem for App {
    fn add_engine_tick_systems<M>(&mut self, systems: impl IntoSystemConfigs<M>) -> &mut Self {
        self.add_systems(FixedUpdate, systems.run_if(game_running))
    }
}

pub fn setup(mut commands: Commands) {
    // Ground plane
    const WALL: f32 = 1.0;
    const HALF_WALL: f32 = WALL * 2.0;
    const HALF_PLANE: f32 = PLANE * 0.5;
    let collider = Collider::compound(vec![
        (
            Vec3::new(-HALF_WALL, -HALF_PLANE - HALF_WALL, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(HALF_PLANE + HALF_WALL, HALF_WALL, HALF_WALL),
        ),
        (
            Vec3::new(-HALF_PLANE - HALF_WALL, HALF_WALL, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(HALF_WALL, HALF_PLANE + HALF_WALL, HALF_WALL),
        ),
        (
            Vec3::new(HALF_WALL, HALF_PLANE + HALF_WALL, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(HALF_PLANE + HALF_WALL, HALF_WALL, HALF_WALL),
        ),
        (
            Vec3::new(HALF_PLANE + HALF_WALL, -HALF_WALL, 0.0),
            Quat::IDENTITY,
            Collider::cuboid(HALF_WALL, HALF_PLANE + HALF_WALL, HALF_WALL),
        ),
    ]);
    commands.spawn((RigidBody::KinematicPositionBased, collider));
}

// Returns an angle of rotation, along the z-axis, so that `from` will be pointing to `to`
pub fn pointing_angle(from: Vec3, to: Vec3) -> f32 {
    let dir = to - from;
    -dir.truncate().angle_between(Vec2::Y)
}
