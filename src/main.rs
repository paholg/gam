use bevy::{
    input::mouse::MouseMotion,
    prelude::{
        default, shape, App, AssetServer, Assets, Bundle, Camera, Camera3d, Camera3dBundle, Color,
        Commands, Component, EventReader, GlobalTransform, Image, Input, KeyCode, Material, Mesh,
        OrthographicProjection, PbrBundle, PointLightBundle, Query, Res, ResMut, StandardMaterial,
        Transform, Vec2, Vec3, With,
    },
    render::camera::ScalingMode,
    sprite::ColorMaterial,
    window::Windows,
    DefaultPlugins,
};

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Direction {
    phi: f32,
}

#[derive(Component)]
struct Health {
    cur: f32,
    max: f32,
}

#[derive(Component)]
struct Radius {
    r: f32,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Bundle)]
struct CharacterBundle {
    pos: Position,
    dir: Direction,
    health: Health,
    radius: Radius,
}

const PLAYER_R: f32 = 1.0;

const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, -50.0, 50.0);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn()
        .insert_bundle(CharacterBundle {
            pos: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            dir: Direction { phi: 0.0 },
            health: Health {
                cur: 100.0,
                max: 100.0,
            },
            radius: Radius { r: PLAYER_R },
        })
        .insert_bundle(PbrBundle {
            mesh: meshes.add(
                shape::Icosphere {
                    radius: 1.0,
                    ..default()
                }
                .into(),
            ),
            material: materials.add(Color::LIME_GREEN.into()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(PLAYER_R, PLAYER_R, PLAYER_R),
                ..default()
            },
            ..default()
        })
        .insert(Player);

    // ground plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(
            shape::Quad {
                size: Vec2::new(20.0, 20.0),
                ..default()
            }
            .into(),
        ),
        material: materials.add(Color::SILVER.into()),
        ..default()
    });

    commands.spawn_bundle(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 10.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(0.0, 0.0, 5.0),
        ..default()
    });
}

fn move_player(input: Res<Input<KeyCode>>, mut query: Query<&mut Transform, With<Player>>) {
    const SPEED: f32 = 0.1;
    let mut transform = query.single_mut();
    let mut vec = Vec2::new(0.0, 0.0);
    if input.pressed(KeyCode::S) {
        vec += Vec2::new(-SPEED, 0.0);
    }
    if input.pressed(KeyCode::F) {
        vec += Vec2::new(SPEED, 0.0);
    }
    if input.pressed(KeyCode::E) {
        vec += Vec2::new(0.0, SPEED);
    }
    if input.pressed(KeyCode::D) {
        vec += Vec2::new(0.0, -SPEED);
    }
    vec = vec.clamp_length_max(SPEED);

    let delta = Vec3::new(vec.x, vec.y, 0.0);
    transform.translation += delta;
}

fn move_camera(
    windows: Res<Windows>,
    mut mouse: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
) {
    // Assume one camera
    let (transform, camera, global_transform) = query.single();

    let window = windows.get_primary().unwrap();
    let pos = match window.cursor_position() {
        Some(pos) => pos,
        None => return,
    };
    let window_size = Vec2::new(window.width(), window.height());
    // convert screen position ot ndc (gpu coords)
    let ndc = (pos / window_size) * 2.00 - Vec2::ONE;
    // Matrix for undoing the projection and camera transform
    let ndc_to_world = transform.compute_matrix() * camera.projection_matrix().inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    // let world_pos: Vec2 = world_pos.truncate();

    println!("{world_pos}");
}

fn main() {
    App::new()
        .add_startup_system(setup)
        .add_system(move_player)
        .add_system(move_camera)
        .add_plugins(DefaultPlugins)
        .run();
}
