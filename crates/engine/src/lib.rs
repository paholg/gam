pub mod ability;
pub mod ai;
pub mod collision;
pub mod death_callback;
pub mod input;
pub mod lifecycle;
pub mod multiplayer;
pub mod physics;
pub mod player;
pub mod status_effect;
pub mod time;

use std::fmt;

use ability::{
    grenade::GrenadeLandEvent, properties::AbilityProps, seeker_rocket, Abilities, Ability,
};
use bevy_app::{App, FixedUpdate, Plugin, PostUpdate, Startup};
use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    schedule::{IntoSystemConfigs, IntoSystemSetConfigs, State, States, SystemSet},
    system::{Commands, Query, Res, Resource},
};
use bevy_math::{Quat, Vec2, Vec3};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalImpulse, LockedAxes, ReadMassProperties,
    RigidBody, Velocity,
};
use bevy_reflect::Reflect;
use bevy_time::{Fixed, Time};
use bevy_transform::components::{GlobalTransform, Transform};
use bevy_utils::HashMap;
use input::check_resume;
use lifecycle::DeathEvent;
use multiplayer::PlayerInputs;
use physics::PhysicsPlugin;
use status_effect::StatusEffects;
use time::{Tick, TickCounter, FREQUENCY};

#[derive(States, PartialEq, Eq, Debug, Copy, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
    Paused,
}

pub const PLAYER_R: f32 = 1.0;
const IMPULSE: f32 = 15.0;
// TODO: Replace this with friction maybe?
// That might make it easier to have slippery/sticky ground effects.
const DAMPING: Damping = Damping {
    linear_damping: 5.0,
    angular_damping: 0.0,
};

/// Represents the kind of entity this is; used, at least, for effects.
///
/// All in-game entities should probably have this component.
#[derive(Component, Debug, Default, Copy, Clone)]
pub enum Kind {
    #[default]
    Other,
    Player,
    Enemy,
    Ally,
    Bullet,
    FragGrenade,
    HealGrenade,
    SeekerRocket,
}

pub const PLANE: f32 = 50.0;

#[derive(Component, Default, Reflect, Debug)]
pub struct Health {
    pub cur: f32,
    pub max: f32,
    pub death_delay: Tick,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self::new_with_delay(max, Tick(0))
    }

    pub fn new_with_delay(max: f32, death_delay: Tick) -> Self {
        Self {
            cur: max,
            max,
            death_delay,
        }
    }

    pub fn take(&mut self, dmg: f32) {
        // Note: Damage can be negative (for healing) so we need to clamp by
        // both min (0) and max.
        self.cur = (self.cur - dmg).clamp(0.0, self.max);
    }

    pub fn die(&mut self) {
        self.cur = 0.0;
    }
}

#[derive(Component, Default, Reflect)]
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

/// We currently move Characters by applying an impulse; this is the highest
/// impulse they can use.
#[derive(Component, Copy, Clone, Debug, Reflect)]
pub struct MaxSpeed {
    pub impulse: f32,
}

impl Default for MaxSpeed {
    fn default() -> Self {
        Self { impulse: IMPULSE }
    }
}

/// A target corresponds to a player's cursor location in game coordinates.
/// It may also end up representing something for AI.
#[derive(Component, Default)]
pub struct Target(pub Vec2);

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash, Resource)]
pub struct Player {
    handle: u32,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player({})", self.handle)
    }
}

impl Player {
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }
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

#[derive(Component, Reflect, Default)]
pub struct Cooldowns {
    map: HashMap<Ability, Tick>,
}

impl Cooldowns {
    pub fn new(abilities: &Abilities) -> Self {
        let cooldowns = abilities
            .iter()
            .map(|ability| (ability, Tick::default()))
            .collect();
        Self { map: cooldowns }
    }

    pub fn get_mut(&mut self, ability: &Ability) -> Option<&mut Tick> {
        self.map.get_mut(ability)
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
    kind: Kind,
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
    abilities: Abilities,
}

#[derive(Resource, Reflect, Default, Debug)]
pub struct NumAi {
    pub enemies: usize,
    pub allies: usize,
}

/// This plugin contains everything needed to run the game headlessly.
pub struct GamPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Timer,
    Input,
    Ai,
    Physics1,
    Physics2,
    Physics3,
    Stuff,
    Despawn,
}

impl Plugin for GamPlugin {
    fn build(&self, app: &mut App) {
        // State
        app.add_state::<AppState>();

        // Resources
        app.insert_resource(Time::<Fixed>::from_hz(FREQUENCY as f64))
            .insert_resource(TickCounter::new())
            .insert_resource(AbilityProps::default())
            .insert_resource(NumAi {
                enemies: 0,
                allies: 0,
            })
            .insert_resource(PlayerInputs::default());

        // Events
        // TODO: Currently, these events are only used for engine -> client
        // communication. We should probably come up with a method so that the
        // server does not need to generate them.
        // Note: If any events are needed by the server, don't use `add_event`. See
        // https://bevy-cheatbook.github.io/patterns/manual-event-clear.html
        app.add_event::<GrenadeLandEvent>()
            .add_event::<DeathEvent>();

        let physics = PhysicsPlugin::new();

        let schedule = FixedUpdate;

        // Sytem sets
        app.configure_sets(
            schedule.clone(),
            (
                GameSet::Timer,
                GameSet::Input,
                GameSet::Ai,
                GameSet::Physics1,
                GameSet::Physics2,
                GameSet::Physics3,
                GameSet::Stuff,
                GameSet::Despawn,
            )
                .chain(),
        );

        // Systems in order
        app.add_systems(Startup, setup).add_systems(
            schedule.clone(),
            (
                (time::tick_counter, time::debug_tick_system).in_set(GameSet::Timer),
                (input::apply_inputs).in_set(GameSet::Input),
                (ai::simple::system_set()).in_set(GameSet::Ai),
                physics.set1().in_set(GameSet::Physics1),
                physics.set2().in_set(GameSet::Physics2),
                physics.set3().in_set(GameSet::Physics3),
                (
                    // Note: Most things should go here.
                    energy_regen,
                    lifecycle::reset,
                    ability::grenade::grenade_land_system,
                    ability::bullet::bullet_kickback_system,
                    seeker_rocket::seeker_rocket_tracking,
                    ability::grenade::grenade_explode_system,
                    // Collisions
                    collision::clear_colliding_system,
                    collision::collision_system,
                    ability::bullet::bullet_collision_system,
                    ability::seeker_rocket::seeker_rocket_collision_system,
                    death_callback::explosion_collision_system,
                )
                    .chain()
                    .in_set(GameSet::Stuff),
                (
                    // Put systems that despawn things at the end.
                    ability::bullet::bullet_despawn_system,
                    lifecycle::die,
                )
                    .chain()
                    .in_set(GameSet::Despawn),
            )
                .run_if(game_running),
        );

        // Special pause systems
        app.add_systems(schedule, (check_resume).run_if(game_paused));

        // TODO: This seems to currently be required so rapier does not miss
        // events, but it is likely a source of non-determinism.
        app.add_systems(PostUpdate, (bevy_rapier3d::plugin::systems::sync_removals,));

        // Plugins
        // Note: None of these plugins should include systems; any systems
        // should be included manually below to ensure determinism.
        app.add_plugins(physics);
    }
}

pub fn game_running(state: Res<State<AppState>>) -> bool {
    state.get() == &AppState::Running
}

pub fn game_paused(state: Res<State<AppState>>) -> bool {
    state.get() != &AppState::Running
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

pub fn energy_regen(mut query: Query<&mut Energy>) {
    for mut energy in &mut query {
        energy.cur += energy.regen;
        energy.cur = energy.cur.min(energy.max);
    }
}

/// Rotate the transform so that it faces `target`
///
/// Note: this is different from `Transform::look_at` in that Bevy calls the
/// y-axis up, so our definitions of `forward` and `up` are different.
pub fn face(transform: &mut Transform, target: Vec3) {
    let face = target - transform.translation;
    transform.look_to(-Vec3::Z, face);
}

pub struct PrettyPrinter(String);

impl fmt::Display for PrettyPrinter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for PrettyPrinter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for PrettyPrinter {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// A simple trait for nicely printing types for debugging.
///
/// Mostly used for logging float types with limited precision.
pub trait PrettyPrint {
    fn pp(&self) -> PrettyPrinter;
}

impl PrettyPrint for Vec3 {
    fn pp(&self) -> PrettyPrinter {
        format!("[{:0.1}, {:0.1}, {:0.1}]", self.x, self.y, self.z).into()
    }
}

impl PrettyPrint for Quat {
    fn pp(&self) -> PrettyPrinter {
        format!(
            "[{:0.1}, {:0.1}, {:0.1}, {:0.1}]",
            self.x, self.y, self.z, self.w
        )
        .into()
    }
}

impl PrettyPrint for Vec2 {
    fn pp(&self) -> PrettyPrinter {
        format!("[{:0.1}, {:0.1}]", self.x, self.y).into()
    }
}

impl PrettyPrint for f32 {
    fn pp(&self) -> PrettyPrinter {
        format!("{self:0.1}").into()
    }
}

impl PrettyPrint for Transform {
    fn pp(&self) -> PrettyPrinter {
        format!(
            "<t: {}, r: {}, s: {}>",
            self.translation.pp(),
            self.rotation.pp(),
            self.scale.pp()
        )
        .into()
    }
}

impl PrettyPrint for GlobalTransform {
    fn pp(&self) -> PrettyPrinter {
        self.compute_transform().pp()
    }
}
