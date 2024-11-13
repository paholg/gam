use std::fmt;

use ability::properties::AbilityProps;
use ability::seeker_rocket;
use ability::Abilities;
use ability::Ability;
use ai::pathfind::PathfindPlugin;
use bevy_app::App;
use bevy_app::FixedUpdate;
use bevy_app::Plugin;
use bevy_app::PostUpdate;
use bevy_app::Startup;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::schedule::IntoSystemConfigs;
use bevy_ecs::schedule::IntoSystemSetConfigs;
use bevy_ecs::schedule::SystemSet;
use bevy_ecs::system::Query;
use bevy_ecs::system::Res;
use bevy_ecs::system::Resource;
use bevy_math::prelude::InfinitePlane3d;
use bevy_math::Dir3;
use bevy_math::Quat;
use bevy_math::Vec2;
use bevy_math::Vec3;
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::prelude::ColliderMassProperties;
use bevy_rapier3d::prelude::ExternalForce;
use bevy_rapier3d::prelude::Friction;
use bevy_rapier3d::prelude::LockedAxes;
use bevy_rapier3d::prelude::ReadMassProperties;
use bevy_rapier3d::prelude::RigidBody;
use bevy_rapier3d::prelude::Velocity;
use bevy_reflect::Reflect;
use bevy_state::app::AppExtStates;
use bevy_state::state::State;
use bevy_state::state::States;
use bevy_time::Fixed;
use bevy_time::Time;
use bevy_transform::bundles::TransformBundle;
use bevy_transform::components::GlobalTransform;
use bevy_transform::components::Transform;
use bevy_utils::HashMap;
use collision::TrackCollisionBundle;
use input::check_resume;
use level::InLevel;
use level::LevelProps;
use lifecycle::DeathEvent;
use movement::DesiredMove;
use movement::MaxSpeed;
use multiplayer::PlayerInputs;
use physics::PhysicsPlugin;
use status_effect::charge::charge_tick;
use status_effect::phased::phased_tick;
use status_effect::temperature::temperature_tick;
use status_effect::time_dilation::time_dilation_tick;
use status_effect::StatusBundle;
use status_effect::TimeDilation;
use time::Dur;
use time::FrameCounter;
use time::FREQUENCY;

pub mod ability;
pub mod ai;
pub mod collision;
pub mod death_callback;
pub mod debug;
pub mod input;
pub mod level;
pub mod lifecycle;
pub mod movement;
pub mod multiplayer;
pub mod physics;
pub mod player;
pub mod status_effect;
pub mod time;

pub type Libm = libm::Libm<f32>;

#[derive(States, PartialEq, Eq, Debug, Copy, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
    Paused,
}

pub const FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0);
pub const UP: Vec3 = Vec3::Y;
pub const UP_PLANE: InfinitePlane3d = InfinitePlane3d { normal: Dir3::Y };

pub const PLAYER_R: f32 = 0.25;
pub const PLAYER_HEIGHT: f32 = 0.75;
pub const PLAYER_MASS: f32 = 15.0;
pub const ABILITY_Y: Vec3 = Vec3::new(0.0, 0.4, 0.0);
pub const PLAYER_ABILITY_COUNT: usize = 5;

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
    NeutrinoBall,
    TransportBeam,
}

#[derive(Component, Default, Reflect, Debug)]
pub struct Health {
    pub cur: f32,
    pub max: f32,
    pub death_delay: Dur,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self::new_with_delay(max, Dur::new(0))
    }

    pub fn new_with_delay(max: f32, death_delay: Dur) -> Self {
        Self {
            cur: max,
            max,
            death_delay,
        }
    }

    pub fn take(&mut self, dmg: f32, time_dilation: &TimeDilation) {
        // Note: Damage can be negative (for healing) so we need to clamp by
        // both min (0) and max.
        let damage = dmg * time_dilation.factor();
        self.cur = (self.cur - damage).clamp(0.0, self.max);
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

pub trait Faction: Component {
    type Foe: Component;
}

/// Indicate this entity is on the enemy team.
#[derive(Component)]
pub struct Enemy;

impl Faction for Enemy {
    type Foe = Ally;
}

/// Indicate this entity is on the players' team.
#[derive(Component)]
pub struct Ally;

impl Faction for Ally {
    type Foe = Enemy;
}

/// Indicates that this entity can be hit by shots; think characters and walls.
#[derive(Component, Default)]
pub struct Shootable;

#[derive(Component, Reflect, Default)]
pub struct Cooldowns {
    map: HashMap<Ability, Dur>,
}

impl Cooldowns {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    pub fn is_available(&self, ability: &Ability, time_dilation: &TimeDilation) -> bool {
        match self.map.get(ability) {
            Some(dur) => dur.is_done(time_dilation),
            None => true,
        }
    }

    pub fn set(&mut self, ability: Ability, cooldown: Dur) {
        self.map.insert(ability, cooldown);
    }

    pub fn reset(&mut self) {
        self.map.clear();
    }
}

pub fn cooldown_system(mut query: Query<(&mut Cooldowns, &TimeDilation)>) {
    for (mut cd, time_dilation) in &mut query {
        for (_ability, dur) in cd.map.iter_mut() {
            dur.tick(time_dilation);
        }
        cd.map.retain(|_ability, dur| !dur.is_done(time_dilation));
    }
}

/// The offset from an object's transform, to its bottom.
#[derive(Component, Default, Clone, Copy)]
pub struct FootOffset {
    pub y: f32,
}

impl FootOffset {
    pub fn to_vec(&self) -> Vec3 {
        Vec3::new(0.0, self.y, 0.0)
    }
}

impl From<f32> for FootOffset {
    fn from(y: f32) -> Self {
        Self { y }
    }
}

/// The offset from an object's transform, to where it spawns abilities.
#[derive(Component)]
pub struct AbilityOffset {
    pub y: f32,
}

impl AbilityOffset {
    pub fn to_vec(&self) -> Vec3 {
        Vec3::new(0.0, self.y, 0.0)
    }
}

impl From<f32> for AbilityOffset {
    fn from(y: f32) -> Self {
        Self { y }
    }
}

#[derive(Bundle)]
pub struct MassBundle {
    collider_mass_props: ColliderMassProperties,
    read_mass_props: ReadMassProperties,
}

impl MassBundle {
    pub fn new(mass: f32) -> Self {
        Self {
            collider_mass_props: ColliderMassProperties::Mass(mass),
            read_mass_props: ReadMassProperties::default(),
        }
    }
}

#[derive(Bundle)]
pub struct Object {
    transform: TransformBundle,
    collider: Collider,
    foot_offset: FootOffset,
    mass: MassBundle,
    body: RigidBody,
    velocity: Velocity,
    force: ExternalForce,
    locked_axes: LockedAxes,
    kind: Kind,
    in_level: InLevel,
    statuses: StatusBundle,
    collisions: TrackCollisionBundle,
}

#[derive(Component)]
pub struct CharacterMarker;

#[derive(Bundle)]
struct Character {
    object: Object,
    health: Health,
    energy: Energy,
    max_speed: MaxSpeed,
    friction: Friction,
    shootable: Shootable,
    cooldowns: Cooldowns,
    abilities: Abilities,
    desired_movement: DesiredMove,
    ability_offset: AbilityOffset,
    marker: CharacterMarker,
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
        app.init_state::<AppState>();

        // Resources
        app.insert_resource(Time::<Fixed>::from_hz(FREQUENCY as f64))
            .insert_resource(FrameCounter::new())
            .insert_resource(AbilityProps::default())
            .insert_resource(NumAi {
                enemies: 0,
                allies: 0,
            })
            .insert_resource(PlayerInputs::default())
            .insert_resource(LevelProps::default());

        // Events
        // TODO: Currently, these events are only used for engine -> client
        // communication. We should probably come up with a method so that the
        // server does not need to generate them.
        // Note: If any events are needed by the server, don't use `add_event`. See
        // https://bevy-cheatbook.github.io/patterns/manual-event-clear.html
        app.add_event::<DeathEvent>();

        let physics = PhysicsPlugin::new();

        let schedule = FixedUpdate;

        // Sytem sets
        app.configure_sets(
            schedule.clone(),
            (
                GameSet::Timer,
                GameSet::Physics1,
                GameSet::Physics2,
                GameSet::Physics3,
                GameSet::Stuff,
                GameSet::Despawn,
            )
                .chain(),
        );

        // Systems in order
        app.add_systems(Startup, level::test_level).add_systems(
            schedule.clone(),
            (
                time::frame_counter.in_set(GameSet::Timer),
                (
                    physics.set1().in_set(GameSet::Physics1),
                    physics.set2().in_set(GameSet::Physics2),
                    physics.set3().in_set(GameSet::Physics3),
                ),
                (
                    // Note: Most things should go here.
                    (
                        // Initial frame resets.
                        temperature_tick,
                        charge_tick,
                        time_dilation_tick,
                        phased_tick,
                        clear_forces,
                        energy_regen,
                        cooldown_system,
                    )
                        .chain(),
                    (
                        // Entities spawn with 0 mass, so we need to place this
                        // after we run physics, after firing the bullet.
                        // https://github.com/dimforge/bevy_rapier/issues/484
                        ability::bullet::kickback_system,
                    )
                        .chain(),
                    (
                        // Inputs
                        input::apply_inputs,
                        ai::pathfind::poll_pathfinding_system,
                        ai::charge::system_set(),
                    )
                        .chain(),
                    (
                        // Misc; categorize futher?
                        movement::apply_movement,
                        seeker_rocket::tracking_system,
                        ability::grenade::explode_system,
                        death_callback::explosion_grow_system,
                        ability::neutrino_ball::activation_system,
                        ability::transport::move_system,
                        ability::transport::activation_system,
                        lifecycle::fall,
                        ai::pathfind::pathfinding_system,
                    )
                        .chain(),
                    (
                        // Collisions
                        collision::collision_system,
                        ability::bullet::collision_system,
                        ability::seeker_rocket::collision_system,
                        ability::neutrino_ball::collision_system,
                        ability::transport::collision_system,
                        death_callback::explosion_collision_system,
                    )
                        .chain(),
                )
                    .chain()
                    .in_set(GameSet::Stuff),
                (
                    // Put systems that despawn things at the end.
                    ability::bullet::despawn_system,
                    lifecycle::die,
                    lifecycle::reset,
                    time::debug_frame_system,
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
        // should be included manually to ensure determinism.
        // TODO: The `ChargeAiPlugin` does include systems, that run on `Update`. We'll
        // need to patch oxidized_navigation or use something else.`
        app.add_plugins(physics).add_plugins(PathfindPlugin);
    }
}

pub fn game_running(state: Res<State<AppState>>) -> bool {
    state.get() == &AppState::Running
}

pub fn game_paused(state: Res<State<AppState>>) -> bool {
    state.get() != &AppState::Running
}

pub fn energy_regen(mut query: Query<(&mut Energy, &TimeDilation)>) {
    for (mut energy, dilation) in &mut query {
        energy.cur += energy.regen * dilation.factor();
        energy.cur = energy.cur.min(energy.max);
    }
}

pub fn clear_forces(mut query: Query<&mut ExternalForce>) {
    for mut force in &mut query {
        *force = ExternalForce::default();
    }
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

/// Orient a transform to look at the target, being careful to keep its
/// orientation in the plane.
pub fn face(transform: &mut Transform, target: Vec2) {
    let y = transform.translation.y;
    transform.look_at(target.to_3d(y), UP);
    debug_assert!(
        transform.is_finite(),
        "transform '{transform:?}' NaN while trying to face '{target:?}'"
    );
}

/// Sometimes we want to work in a 2d plane, so functions like `Vec3::truncate`
/// and `Vec2::extend` would be useful, except that Bevy and Rapier really want
/// us to consider Y to be up, so the XZ plane in 3d becomes the XY plane in 2d.
pub trait To2d {
    fn to_2d(self) -> Vec2;
}

impl To2d for Vec3 {
    fn to_2d(self) -> Vec2 {
        Vec2::new(self.x, -self.z)
    }
}

/// Sometimes we want to work in a 2d plane, so functions like `Vec3::truncate`
/// and `Vec2::extend` would be useful, except that Bevy and Rapier really want
/// us to consider Y to be up, so the XZ plane in 3d becomes the XY plane in 2d.
pub trait To3d {
    fn to_3d(self, y: f32) -> Vec3;
}

impl To3d for Vec2 {
    fn to_3d(self, y: f32) -> Vec3 {
        Vec3::new(self.x, y, -self.y)
    }
}
