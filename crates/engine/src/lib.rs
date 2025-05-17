use std::fmt;

use ability::cooldown::global_cooldown_tick_system;
use ability::cooldown::Cooldown;
use ability::AbilityMap;
use ability::AbilityPlugin;
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
use bevy_rapier3d::prelude::ContactSkin;
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
use bevy_transform::components::GlobalTransform;
use bevy_transform::components::Transform;
use collision::TrackCollisionBundle;
use input::pause_resume;
use level::InLevel;
use level::LevelProps;
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
    // FIXME: make default again
    Loading,
    #[default]
    Running,
    Menu,
}
pub const SCHEDULE: FixedUpdate = FixedUpdate;

pub const FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0);
pub const UP: Vec3 = Vec3::Y;
pub const UP_PLANE: InfinitePlane3d = InfinitePlane3d { normal: Dir3::Y };

pub const PLAYER_R: f32 = 0.25;
pub const PLAYER_HEIGHT: f32 = 0.75;
pub const PLAYER_MASS: f32 = 15.0;
pub const ABILITY_Y: Vec3 = Vec3::new(0.0, 0.4, 0.0);
pub const PLAYER_ABILITY_COUNT: usize = 5;
pub const CONTACT_SKIN: ContactSkin = ContactSkin(0.01);

#[derive(Component, Default, Reflect, Debug)]
pub struct Health {
    pub cur: f32,
    pub max: f32,
    // This prevents death, ticking every frame below 0 heath. It was added to
    // have some abilities spawn things that can't die.
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

#[derive(Component, Default, Debug, Reflect)]
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
    transform: Transform,
    collider: Collider,
    foot_offset: FootOffset,
    mass: MassBundle,
    body: RigidBody,
    velocity: Velocity,
    force: ExternalForce,
    locked_axes: LockedAxes,
    in_level: InLevel,
    statuses: StatusBundle,
    collisions: TrackCollisionBundle,
}

#[derive(Component)]
pub struct CharacterMarker;

#[derive(Bundle)]
struct Character {
    object: Object,
    contact_skin: ContactSkin,
    health: Health,
    energy: Energy,
    max_speed: MaxSpeed,
    friction: Friction,
    shootable: Shootable,
    global_cooldown: Cooldown,
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
    Reset,
    Input,
    Ai,
    Collision,
    Stuff,
    Physics1,
    Physics2,
    Physics3,
    Despawn,
}

impl Plugin for GamPlugin {
    fn build(&self, app: &mut App) {
        // State
        app.init_state::<AppState>();

        // Resources
        app.insert_resource(Time::<Fixed>::from_hz(FREQUENCY as f64))
            .insert_resource(FrameCounter::new())
            .insert_resource(NumAi {
                enemies: 0,
                allies: 0,
            })
            .insert_resource(PlayerInputs::default())
            .insert_resource(LevelProps::default())
            .init_resource::<AbilityMap>();

        let physics = PhysicsPlugin::new();

        // Sytem sets
        app.configure_sets(
            SCHEDULE,
            (
                GameSet::Timer,
                GameSet::Reset,
                GameSet::Input,
                GameSet::Ai,
                GameSet::Collision,
                GameSet::Stuff,
                GameSet::Physics1,
                GameSet::Physics2,
                GameSet::Physics3,
                GameSet::Despawn,
            )
                .chain()
                .run_if(game_running),
        );

        // Systems in order
        app.add_systems(Startup, level::test_level).add_systems(
            SCHEDULE,
            (
                time::frame_counter.in_set(GameSet::Timer),
                (
                    time_dilation_tick,
                    temperature_tick,
                    charge_tick,
                    phased_tick,
                    clear_forces,
                    energy_regen,
                    global_cooldown_tick_system,
                    collision::collision_system,
                )
                    .in_set(GameSet::Reset),
                input::apply_inputs.in_set(GameSet::Input),
                ai::systems().in_set(GameSet::Ai),
                (ability::bullet::collision_system,)
                    .chain()
                    .in_set(GameSet::Collision),
                (
                    // Misc; categorize futher?
                    movement::apply_movement,
                    // death_callback::explosion_grow_system,
                    lifecycle::fall,
                )
                    .in_set(GameSet::Stuff),
                (
                    physics.set1().in_set(GameSet::Physics1),
                    physics.set2().in_set(GameSet::Physics2),
                    physics.set3().in_set(GameSet::Physics3),
                ),
                (
                    lifecycle::lifetime_system,
                    lifecycle::die,
                    lifecycle::reset,
                    time::debug_frame_system,
                    // Entities spawn with 0 mass, so we need to place this
                    // after we run physics, after firing the bullet.
                    // https://github.com/dimforge/bevy_rapier/issues/484
                    ability::bullet::kickback_system,
                )
                    .chain()
                    .in_set(GameSet::Despawn),
            ),
        );

        // TODO: This is a potential source of non-determinism. I suspect that it will run either
        // before or after other systems, determined at launch time, which is not what we want.
        app.add_systems(SCHEDULE, pause_resume);

        // Ability Plugins
        app.add_plugins(AbilityPlugin);

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
