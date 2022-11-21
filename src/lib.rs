#![feature(
    once_cell,
    duration_consts_float,
    div_duration,
    const_fn_floating_point_arithmetic
)]
#![allow(clippy::type_complexity)]

pub mod ability;
pub mod ai;
pub mod config;
pub mod healthbar;
pub mod physics;
pub mod system;
pub mod time;

use bevy::{
    app::PluginGroupBuilder,
    audio::AudioPlugin,
    diagnostic::DiagnosticsPlugin,
    gltf::GltfPlugin,
    input::InputPlugin,
    log::LogPlugin,
    pbr::PbrPlugin,
    prelude::{
        default, shape, AnimationPlugin, App, AssetPlugin, Assets, Bundle, Camera, Camera3dBundle,
        Color, Commands, Component, ComputedVisibility, CorePlugin, GilrsPlugin, GlobalTransform,
        Handle, HierarchyPlugin, ImagePlugin, IntoSystemDescriptor, Mat4, Mesh,
        OrthographicProjection, PbrBundle, Plugin, PluginGroup, PointLight, PointLightBundle,
        Query, Ray, Res, ResMut, Resource, StandardMaterial, SystemSet, Transform, Vec2, Vec3,
        Visibility,
    },
    render::{camera::ScalingMode, RenderPlugin},
    scene::{Scene, ScenePlugin},
    sprite::SpritePlugin,
    text::TextPlugin,
    time::TimePlugin,
    transform::TransformPlugin,
    ui::UiPlugin,
    window::{WindowPlugin, Windows},
    winit::WinitPlugin,
};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RigidBody, Velocity};
use healthbar::HealthbarPlugin;
use iyes_loopless::{fixedtimestep::TimestepName, prelude::AppLooplessFixedTimestepExt};
use physics::PhysicsPlugin;
use time::{Tick, TIMESTEP};

use crate::healthbar::Healthbar;

#[derive(Component)]
pub struct Health {
    cur: f32,
    max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { cur: max, max }
    }
}

#[derive(Component, Copy, Clone, Debug)]
pub struct MaxSpeed(f32);

/// Indicate this entity is a player. Currently, we assume one player.
#[derive(Component)]
pub struct Player;

/// Indicate this entity is controlled by AI.
#[derive(Component)]
pub struct Ai;

/// Indicate this entity is on the enemy team.
#[derive(Component)]
pub struct Enemy;

/// Indicate this entity is on the players' team.
#[derive(Component)]
pub struct Ally;

// TODO: Do cooldowns better. We don't want every entity to have a giant
// cooldowns struct.
#[derive(Component)]
pub struct Cooldowns {
    hyper_sprint: Tick,
    shoot: Tick,
}

pub fn cooldown_system(mut cooldowns: Query<&mut Cooldowns>) {
    for mut cd in cooldowns.iter_mut() {
        cd.hyper_sprint.tick();
        cd.shoot.tick();
    }
}

#[derive(Bundle)]
pub struct Object {
    #[cfg(feature = "graphics")]
    material: Handle<StandardMaterial>,
    #[cfg(feature = "graphics")]
    mesh: Handle<Mesh>,
    transform: Transform,
    global_transform: GlobalTransform,
    #[cfg(feature = "graphics")]
    visibility: Visibility,
    #[cfg(feature = "graphics")]
    computed_visibility: ComputedVisibility,
    collider: Collider,
    body: RigidBody,
    velocity: Velocity,
    locked_axes: LockedAxes,
}

#[derive(Bundle)]
struct Character {
    health: Health,
    #[cfg(feature = "graphics")]
    healthbar: Healthbar,
    #[cfg(feature = "graphics")]
    scene: Handle<Scene>,
    #[cfg(feature = "graphics")]
    outline: Handle<Mesh>,
    #[cfg(feature = "graphics")]
    material: Handle<StandardMaterial>,
    transform: Transform,
    global_transform: GlobalTransform,
    #[cfg(feature = "graphics")]
    visibility: Visibility,
    #[cfg(feature = "graphics")]
    computed_visibility: ComputedVisibility,
    collider: Collider,
    body: RigidBody,
    max_speed: MaxSpeed,
    velocity: Velocity,
    locked_axes: LockedAxes,
}

const PLAYER_R: f32 = 1.0;
const SPEED: f32 = 15.0;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

pub const PLANE_SIZE: f32 = 30.0;

#[derive(Resource)]
pub struct NumAi {
    pub enemies: u32,
    pub allies: u32,
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
            .add(CorePlugin::default())
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
            .add(AudioPlugin::default())
            .add(GilrsPlugin::default())
            .add(AnimationPlugin::default());
        group
    }
}

/// This plugin contains everything needed to run the game headlessly.
pub struct GamPlugin;

impl Plugin for GamPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_fixed_timestep(TIMESTEP, BEFORE_CORESTAGE_UPDATE)
            .insert_resource(NumAi {
                enemies: 5,
                allies: 5,
            })
            .add_startup_system(setup)
            .add_engine_tick_system(system::die)
            .add_engine_tick_system(system::reset)
            .add_engine_tick_system(ability::hyper_sprint_system)
            .add_engine_tick_system(ability::shot_despawn_system)
            .add_engine_tick_system(ability::shot_hit_system_ally)
            .add_engine_tick_system(ability::shot_hit_system_enemy)
            .add_engine_tick_system(ability::shot_miss_system)
            .add_engine_tick_system(cooldown_system)
            .add_plugin(ai::simple::SimpleAiPlugin)
            .add_plugin(ai::qlearning::QLearningPlugin)
            .add_plugin(PhysicsPlugin);
    }
}

/// This plugin includes user input and graphics.
pub struct GamClientPlugin;

impl Plugin for GamClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_engine_tick_system(system::player_input)
            .add_engine_tick_system(system::update_cursor)
            .add_plugin(HealthbarPlugin);
    }
}

/// A helper enum to make tracking our custom Fixedtimestep stages a bit more
/// sane.
pub enum CustomStage {
    CoreUpdate,
    // The Physics stages are added by the PhysicsPlugin, based on the order in
    // bevy_rapier's RapierPhysicsPlugin::build().
    PhysicsSyncBackend,
    PhysicsStepSimulation,
    PhysicsWriteback,
    PhysicsDetectDespawn,
}

pub const BEFORE_CORESTAGE_UPDATE: &str = "Before CoreStage::Update";
pub const AFTER_CORESTAGE_UPDATE: &str = "After CoreStage::Update";
pub const BEFORE_CORESTAGE_LAST: &str = "Before CoreStage::Last";

impl CustomStage {
    pub fn timestep_name(&self) -> TimestepName {
        match self {
            CustomStage::CoreUpdate => BEFORE_CORESTAGE_UPDATE,
            CustomStage::PhysicsSyncBackend => AFTER_CORESTAGE_UPDATE,
            CustomStage::PhysicsStepSimulation => AFTER_CORESTAGE_UPDATE,
            CustomStage::PhysicsWriteback => AFTER_CORESTAGE_UPDATE,
            CustomStage::PhysicsDetectDespawn => BEFORE_CORESTAGE_LAST,
        }
    }

    pub fn substage(&self) -> usize {
        match self {
            CustomStage::CoreUpdate => 0,
            CustomStage::PhysicsSyncBackend => 0,
            CustomStage::PhysicsStepSimulation => 1,
            CustomStage::PhysicsWriteback => 2,
            CustomStage::PhysicsDetectDespawn => 0,
        }
    }
}

trait FixedTimestepSystem {
    fn add_engine_tick_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self;

    fn add_engine_tick_system_to_stage<Params>(
        &mut self,
        stage: CustomStage,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self;

    fn add_engine_tick_system_set_to_stage(
        &mut self,
        stage: CustomStage,
        system_set: SystemSet,
    ) -> &mut Self;
}

impl FixedTimestepSystem for App {
    fn add_engine_tick_system<Params>(
        &mut self,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.add_fixed_timestep_system(
            CustomStage::CoreUpdate.timestep_name(),
            CustomStage::CoreUpdate.substage(),
            system,
        )
    }
    fn add_engine_tick_system_to_stage<Params>(
        &mut self,
        stage: CustomStage,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        self.add_fixed_timestep_system(stage.timestep_name(), stage.substage(), system)
    }

    fn add_engine_tick_system_set_to_stage(
        &mut self,
        stage: CustomStage,
        system_set: SystemSet,
    ) -> &mut Self {
        self.add_fixed_timestep_system_set(stage.timestep_name(), stage.substage(), system_set)
    }
}

pub fn setup(
    mut commands: Commands,
    #[cfg(feature = "graphics")] mut meshes: ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    let collider = Collider::compound(vec![
        (
            Vec2::new(PLANE_SIZE * 1.5, 0.0),
            0.0,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE),
        ),
        (
            Vec2::new(-PLANE_SIZE * 1.5, 0.0),
            0.0,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE),
        ),
        (
            Vec2::new(0.0, PLANE_SIZE * 1.5),
            0.0,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE),
        ),
        (
            Vec2::new(0.0, -PLANE_SIZE * 1.5),
            0.0,
            Collider::cuboid(PLANE_SIZE, PLANE_SIZE),
        ),
    ]);
    let ground_plane = commands
        .spawn((RigidBody::KinematicPositionBased, collider))
        .id();
    #[cfg(feature = "graphics")]
    commands.entity(ground_plane).insert(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(PLANE_SIZE, PLANE_SIZE),
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
    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 10.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // Light
    #[cfg(feature = "graphics")]
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            range: PLANE_SIZE,
            intensity: 2000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 5.0),
        ..default()
    });
}

// Logic taken from here:
// https://github.com/lucaspoffo/renet/blob/c963b65b66325c536d115faab31638f3ad2b5e48/demo_bevy/src/lib.rs#L196-L215
fn ray_from_screenspace(
    windows: &Res<Windows>,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Ray> {
    let window = windows.get_primary().unwrap();
    let cursor_position = window.cursor_position()?;

    let view = camera_transform.compute_matrix();
    let screen_size = camera.logical_target_size()?;
    let projection = camera.projection_matrix();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (cursor_position / screen_size) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = far - near;

    Some(Ray {
        origin: near,
        direction: ray_direction,
    })
}

// Logic taken from here:
// https://github.com/lucaspoffo/renet/blob/c963b65b66325c536d115faab31638f3ad2b5e48/demo_bevy/src/lib.rs#L217-L228
fn intersect_xy_plane(ray: &Ray, z_offset: f32) -> Option<Vec3> {
    let plane_normal = Vec3::Z;
    let plane_origin = Vec3::new(0.0, z_offset, 0.0);
    let denominator = ray.direction.dot(plane_normal);
    if denominator.abs() > f32::EPSILON {
        let point_to_point = plane_origin * z_offset - ray.origin;
        let intersect_dist = plane_normal.dot(point_to_point) / denominator;
        let intersect_position = ray.direction * intersect_dist + ray.origin;
        Some(intersect_position)
    } else {
        None
    }
}

// Returns an angle of rotation, along the z-axis, so that `from` will be pointing to `to`
fn pointing_angle(from: Vec3, to: Vec3) -> f32 {
    let dir = to - from;
    let angle = dir.angle_between(Vec3::Y);
    if dir.x > 0.0 {
        angle * -1.0
    } else {
        angle
    }
}
