#![feature(
    duration_consts_float,
    div_duration,
    const_fn_floating_point_arithmetic,
    io_error_other
)]
#![allow(clippy::type_complexity)]

pub mod ability;
pub mod ai;
pub mod client;
pub mod physics;
pub mod shapes;
pub mod status_effect;
pub mod system;
pub mod time;

use std::{f32::consts::PI, time::Duration};

use ability::ShotHitEvent;
use bevy::{
    app::PluginGroupBuilder,
    core_pipeline::bloom::BloomSettings,
    diagnostic::DiagnosticsPlugin,
    gltf::GltfPlugin,
    input::InputPlugin,
    log::LogPlugin,
    pbr::PbrPlugin,
    prelude::{
        default, shape, AnimationPlugin, App, AssetPlugin, Assets, Bundle, Camera, Camera3dBundle,
        Color, Commands, Component, CoreSchedule, FixedTime, FrameCountPlugin, GilrsPlugin,
        GlobalTransform, HierarchyPlugin, ImagePlugin, IntoSystemAppConfig, IntoSystemConfig, Mesh,
        PbrBundle, PerspectiveProjection, Plugin, PluginGroup, PointLight, PointLightBundle, Quat,
        Res, ResMut, Resource, StandardMaterial, State, States, TaskPoolPlugin, Transform,
        TypeRegistrationPlugin, Vec2, Vec3,
    },
    render::RenderPlugin,
    scene::ScenePlugin,
    sprite::SpritePlugin,
    text::TextPlugin,
    time::TimePlugin,
    transform::TransformPlugin,
    ui::UiPlugin,
    window::WindowPlugin,
    winit::WinitPlugin,
};
use bevy_rapier3d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalImpulse, LockedAxes, ReadMassProperties,
    RigidBody, Velocity,
};
use physics::PhysicsPlugin;
use status_effect::StatusEffects;
use time::{Tick, TickPlugin, TIMESTEP};

#[derive(States, PartialEq, Eq, Debug, Copy, Clone, Hash, Default)]
pub enum AppState {
    #[cfg_attr(feature = "graphics", default)]
    Loading,
    #[cfg_attr(not(feature = "graphics"), default)]
    Running,
    Paused,
}

pub struct DeathEvent {
    pub transform: Transform,
}

const PLAYER_R: f32 = 1.0;
const IMPULSE: f32 = 15.0;
const DAMPING: Damping = Damping {
    linear_damping: 5.0,
    angular_damping: 0.0,
};

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

pub const PLANE: f32 = 50.0;

#[derive(Component, Default)]
pub struct Health {
    cur: f32,
    max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { cur: max, max }
    }

    pub fn take(&mut self, dmg: f32) {
        self.cur -= dmg;
        self.cur = 0.0f32.max(self.cur);
        self.cur = self.max.min(self.cur);
    }
}

#[derive(Component, Default)]
pub struct Energy {
    cur: f32,
    max: f32,
    regen: f32,
}

impl Energy {
    pub fn new(max: f32, regen: f32) -> Self {
        Self {
            cur: max,
            max,
            regen,
        }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct MaxSpeed {
    impulse: f32,
}

impl Default for MaxSpeed {
    fn default() -> Self {
        Self { impulse: IMPULSE }
    }
}

/// Indicate this entity is a player. Currently, we assume one player.
#[derive(Component)]
pub struct Player {
    target: Vec2,
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

// TODO: Do cooldowns better. We don't want every entity to have a giant
// cooldowns struct.
// Or maybe we do?????
#[derive(Component, Default)]
pub struct Cooldowns {
    shoot: Tick,
    shotgun: Tick,
    frag_grenade: Tick,
    heal_grenade: Tick,
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

#[derive(Bundle, Default)]
struct Character {
    health: Health,
    energy: Energy,
    transform: Transform,
    global_transform: GlobalTransform,
    collider: Collider,
    body: RigidBody,
    max_speed: MaxSpeed,
    velocity: Velocity,
    damping: Damping,
    impulse: ExternalImpulse,
    locked_axes: LockedAxes,
    mass: ReadMassProperties,
    status_effects: StatusEffects,
    shootable: Shootable,
}

#[derive(Resource)]
pub struct NumAi {
    pub enemies: usize,
    pub allies: usize,
}

// TODO: For whatever reason, our PluginGroups based on the DefaultPlugins but
// split into two aren't working. Investigate later.

/// The subset of DefaultPlugins that we want in headless mode.
/// For this to be useful, we'll have to refactor things to only take in assets
/// when we're not in headless mode.
pub struct HeadlessDefaultPlugins;

impl PluginGroup for HeadlessDefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group
            .add(LogPlugin::default())
            .add(TaskPoolPlugin::default())
            .add(TypeRegistrationPlugin::default())
            .add(FrameCountPlugin::default())
            .add(TimePlugin::default())
            .add(TransformPlugin::default())
            .add(HierarchyPlugin::default())
            .add(DiagnosticsPlugin::default());

        group
    }
}

/// The set of DefaultPlugins not included in HeadlessDefaultPlugins.
pub struct ClientDefaultPlugins;

impl PluginGroup for ClientDefaultPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group
            .add(InputPlugin::default())
            .add(WindowPlugin::default())
            .add(AssetPlugin::default())
            // .add(DebugAssetServerPlugin::default())
            .add(ScenePlugin::default())
            .add(WinitPlugin::default())
            .add(RenderPlugin::default())
            .add(ImagePlugin::default())
            .add(SpritePlugin::default())
            .add(TextPlugin::default())
            .add(UiPlugin::default())
            .add(PbrPlugin::default())
            .add(GltfPlugin::default())
            // .add(AudioPlugin::default())
            .add(GilrsPlugin::default())
            .add(AnimationPlugin::default());
        group
    }
}

/// This plugin contains everything needed to run the game headlessly.
pub struct GamPlugin;

impl Plugin for GamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(not(feature = "train"))]
        app.insert_resource(FixedTime::new(Duration::from_secs_f32(TIMESTEP)));
        app.add_state::<AppState>()
            .insert_resource(NumAi {
                enemies: 0,
                allies: 0,
            })
            .add_plugin(TickPlugin)
            .add_startup_system(setup)
            .add_engine_tick_system(ability::hyper_sprint_system)
            .add_engine_tick_system(ability::shot_despawn_system)
            .add_engine_tick_system(ability::grenade::grenade_land_system)
            .add_engine_tick_system(ability::grenade::grenade_explode_system)
            .add_engine_tick_system(ability::grenade::explosion_despawn_system)
            .add_event::<ShotHitEvent>()
            .add_event::<DeathEvent>()
            .add_engine_tick_system(ability::shot_hit_system)
            .add_engine_tick_system(ability::shot_kickback_system)
            .add_plugin(ai::simple::SimpleAiPlugin)
            .add_engine_tick_system(system::die)
            .add_engine_tick_system(system::energy_regen)
            .add_engine_tick_system(system::reset)
            .add_plugin(PhysicsPlugin);
    }
}

trait FixedTimestepSystem {
    fn add_engine_tick_system<M>(&mut self, system: impl IntoSystemAppConfig<M>) -> &mut Self;
}

pub fn game_running(state: Res<State<AppState>>) -> bool {
    state.0 == AppState::Running
}

#[cfg(not(feature = "train"))]
impl FixedTimestepSystem for App {
    fn add_engine_tick_system<M>(&mut self, system: impl IntoSystemAppConfig<M>) -> &mut Self {
        self.add_system(
            system
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(game_running),
        )
    }
}

#[cfg(feature = "train")]
impl FixedTimestepSystem for App {
    fn add_engine_tick_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.add_system(system)
    }
}

pub fn setup(
    mut commands: Commands,
    #[cfg(feature = "graphics")] mut meshes: ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
    let ground_plane = commands
        .spawn((RigidBody::KinematicPositionBased, collider))
        .id();
    #[cfg(feature = "graphics")]
    commands.entity(ground_plane).insert(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(PLANE, PLANE),
                ..default()
            }
            .into(),
        ),
        material: materials.add(Color::SILVER.into()),
        transform: Transform::from_xyz(0.0, 0.0, -0.1),
        ..default()
    });

    // Camera
    #[cfg(feature = "graphics")]
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            projection: PerspectiveProjection {
                fov: PI * 0.125,
                ..default()
            }
            .into(),
            transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        BloomSettings::default(),
    ));

    // Light
    #[cfg(feature = "graphics")]
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            ..default()
        },
        transform: Transform::from_xyz(-0.5 * PLANE, -0.5 * PLANE, 10.0),
        ..default()
    });
    #[cfg(feature = "graphics")]
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.5 * PLANE, -0.5 * PLANE, 10.0),
        ..default()
    });
    #[cfg(feature = "graphics")]
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-0.5 * PLANE, 0.5 * PLANE, 10.0),
        ..default()
    });
    #[cfg(feature = "graphics")]
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.5 * PLANE, 0.5 * PLANE, 10.0),
        ..default()
    });
    #[cfg(feature = "graphics")]
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE,
            intensity: 1000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..default()
    });
}

// Returns an angle of rotation, along the z-axis, so that `from` will be pointing to `to`
fn pointing_angle(from: Vec3, to: Vec3) -> f32 {
    let dir = to - from;
    -dir.truncate().angle_between(Vec2::Y)
}
