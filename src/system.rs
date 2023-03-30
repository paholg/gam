use bevy::{
    prelude::{
        shape, AssetServer, Assets, Camera, Color, Commands, ComputedVisibility,
        DespawnRecursiveExt, Entity, GlobalTransform, Input, KeyCode, Mesh, MouseButton, Quat,
        Query, Res, ResMut, StandardMaterial, Transform, Vec2, Vec3, Visibility, With, Without,
    },
    window::{PrimaryWindow, Window},
};
use bevy_rapier2d::prelude::{Collider, ExternalImpulse, LockedAxes, RigidBody, Velocity};
use rand::Rng;

use crate::{
    ability::{HYPER_SPRINT_COOLDOWN, SHOOT_COOLDOWN},
    config::config,
    healthbar::Healthbar,
    intersect_xy_plane, pointing_angle, ray_from_screenspace,
    time::TickCounter,
    Ai, Ally, Character, Cooldowns, Enemy, Health, MaxSpeed, Player, CAMERA_OFFSET, DAMPING,
    PLANE_SIZE, PLAYER_R,
};

pub fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    #[cfg(feature = "graphics")] mut meshes: ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    tick_counter: Res<TickCounter>,
    mut query: Query<
        (
            Entity,
            &mut Cooldowns,
            &mut Velocity,
            &mut MaxSpeed,
            &mut ExternalImpulse,
            &Transform,
        ),
        With<Player>,
    >,
) {
    let config = config();
    let controls = &config.controls;

    let (entity, mut cooldowns, mut velocity, mut max_speed, mut impulse, transform) =
        match query.get_single_mut() {
            Ok(q) => q,
            Err(_) => return,
        };

    // Abilities:
    let abilities = &config.player.abilities;
    for (control, ability) in controls.abilities.iter().zip(abilities.iter()) {
        if control.pressed(&keyboard_input, &mouse_input) {
            ability.fire(
                &mut commands,
                &tick_counter,
                #[cfg(feature = "graphics")]
                &mut meshes,
                #[cfg(feature = "graphics")]
                &mut materials,
                entity,
                &mut cooldowns,
                &mut max_speed,
                transform,
                &velocity,
            );
        }
    }

    // Movement:
    let mut new_impulse = Vec2::new(0.0, 0.0);

    for (control, dir) in [
        (&controls.left, Vec2::new(-1.0, 0.0)),
        (&controls.right, Vec2::new(1.0, 0.0)),
        (&controls.up, Vec2::new(0.0, 1.0)),
        (&controls.down, Vec2::new(0.0, -1.0)),
    ] {
        if control.pressed(&keyboard_input, &mouse_input) {
            new_impulse += dir;
        }
    }

    new_impulse = new_impulse.clamp_length_max(1.0) * max_speed.impulse;

    impulse.impulse = new_impulse;

    velocity.linvel = velocity.linvel.clamp_length_max(max_speed.max_speed);
}

/// Moves the camera and orients the player based on the mouse cursor.
pub fn update_cursor(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &Camera, &GlobalTransform)>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Camera>)>,
) {
    let (mut camera_transform, camera, global_transform) = camera_query.single_mut();

    let cursor = match ray_from_screenspace(primary_window, camera, global_transform)
        .as_ref()
        .and_then(|ray| intersect_xy_plane(ray, 0.0))
    {
        Some(ray) => ray,
        None => return,
    };

    let player_translation = match player_query.get_single_mut() {
        Ok(mut player_transform) => {
            let angle = pointing_angle(player_transform.translation, cursor);
            player_transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

            player_transform.translation
        }
        Err(_) => {
            // No player; let's just keep things mostly centered for now.
            Vec3::default()
        }
    };

    let camera_weight = 0.9;
    const CURSOR_WEIGHT: f32 = 0.33;
    let look_at = cursor * CURSOR_WEIGHT + player_translation * (1.0 - CURSOR_WEIGHT);
    let look_at = (camera_transform.translation - CAMERA_OFFSET) * camera_weight
        + look_at * (1.0 - camera_weight);

    *camera_transform =
        Transform::from_translation(CAMERA_OFFSET + look_at).looking_at(look_at, Vec3::Z);
}

pub fn die(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.cur <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_player(
    commands: &mut Commands,
    #[cfg(feature = "graphics")] meshes: &mut ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] materials: &mut ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "graphics")] asset_server: &mut Res<AssetServer>,
) {
    commands.spawn((
        Player,
        Ally,
        Character {
            health: Health::new(100.0),
            #[cfg(feature = "graphics")]
            healthbar: Healthbar::default(),
            #[cfg(feature = "graphics")]
            scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
            #[cfg(feature = "graphics")]
            outline: meshes.add(
                shape::Circle {
                    radius: 1.0,
                    vertices: 100,
                }
                .into(),
            ),
            #[cfg(feature = "graphics")]
            material: materials.add(Color::GREEN.into()),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
            #[cfg(feature = "graphics")]
            visibility: Visibility::Visible,
            #[cfg(feature = "graphics")]
            computed_visibility: ComputedVisibility::default(),
            collider: Collider::ball(1.0),
            body: RigidBody::Dynamic,
            max_speed: MaxSpeed::default(),
            velocity: Velocity::default(),
            damping: DAMPING,
            impulse: ExternalImpulse::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
        },
        Cooldowns {
            hyper_sprint: HYPER_SPRINT_COOLDOWN,
            shoot: SHOOT_COOLDOWN,
        },
    ));
}

fn spawn_enemies(
    commands: &mut Commands,
    #[cfg(feature = "graphics")] meshes: &mut ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] materials: &mut ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "graphics")] asset_server: &mut Res<AssetServer>,
    num: usize,
) -> Vec<Vec2> {
    let mut rng = rand::thread_rng();
    let mut locs = Vec::with_capacity(num);
    for _ in 0..num {
        let x = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let loc = Vec2::new(x, y);
        locs.push(loc);
        commands.spawn((
            Enemy,
            Ai,
            Character {
                health: Health::new(100.0),
                #[cfg(feature = "graphics")]
                healthbar: Healthbar::default(),
                #[cfg(feature = "graphics")]
                scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
                #[cfg(feature = "graphics")]
                outline: meshes.add(
                    shape::Circle {
                        radius: 1.0,
                        vertices: 100,
                    }
                    .into(),
                ),
                #[cfg(feature = "graphics")]
                material: materials.add(Color::RED.into()),
                transform: Transform::from_xyz(x, y, 0.0),
                global_transform: GlobalTransform::default(),
                #[cfg(feature = "graphics")]
                visibility: Visibility::Visible,
                #[cfg(feature = "graphics")]
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(1.0),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed::default(),
                velocity: Velocity::default(),
                damping: DAMPING,
                impulse: ExternalImpulse::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
            Cooldowns {
                hyper_sprint: HYPER_SPRINT_COOLDOWN,
                shoot: SHOOT_COOLDOWN,
            },
        ));
    }
    locs
}

fn spawn_allies(
    commands: &mut Commands,
    #[cfg(feature = "graphics")] meshes: &mut ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] materials: &mut ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "graphics")] asset_server: &mut Res<AssetServer>,
    num: usize,
) -> Vec<Vec2> {
    let mut rng = rand::thread_rng();
    let mut locs = Vec::with_capacity(num);
    for _ in 0..num {
        let x = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let y = rng.gen::<f32>() * (PLANE_SIZE - PLAYER_R) - (PLANE_SIZE - PLAYER_R) * 0.5;
        let loc = Vec2::new(x, y);
        locs.push(loc);
        commands.spawn((
            Ally,
            Ai,
            Character {
                health: Health::new(100.0),
                #[cfg(feature = "graphics")]
                healthbar: Healthbar::default(),
                #[cfg(feature = "graphics")]
                scene: asset_server.load("models/temp/craft_speederB.glb#Scene0"),
                #[cfg(feature = "graphics")]
                outline: meshes.add(
                    shape::Circle {
                        radius: 1.0,
                        vertices: 100,
                    }
                    .into(),
                ),
                #[cfg(feature = "graphics")]
                material: materials.add(Color::CYAN.into()),
                transform: Transform::from_xyz(x, y, 0.0),
                global_transform: GlobalTransform::default(),
                #[cfg(feature = "graphics")]
                visibility: Visibility::Visible,
                #[cfg(feature = "graphics")]
                computed_visibility: ComputedVisibility::default(),
                collider: Collider::ball(1.0),
                body: RigidBody::Dynamic,
                max_speed: MaxSpeed::default(),
                velocity: Velocity::default(),
                damping: DAMPING,
                impulse: ExternalImpulse::default(),
                locked_axes: LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
            },
            Cooldowns {
                hyper_sprint: HYPER_SPRINT_COOLDOWN,
                shoot: SHOOT_COOLDOWN,
            },
        ));
    }
    locs
}

pub fn reset(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    ally_query: Query<Entity, With<Ally>>,
    player_query: Query<Entity, With<Player>>,
    #[cfg(feature = "graphics")] mut meshes: ResMut<Assets<Mesh>>,
    #[cfg(feature = "graphics")] mut materials: ResMut<Assets<StandardMaterial>>,
    #[cfg(feature = "graphics")] mut asset_server: Res<AssetServer>,
) {
    if enemy_query.iter().next().is_none() {
        spawn_enemies(
            &mut commands,
            #[cfg(feature = "graphics")]
            &mut meshes,
            #[cfg(feature = "graphics")]
            &mut materials,
            #[cfg(feature = "graphics")]
            &mut asset_server,
            1,
        );
    }

    #[cfg(not(feature = "train"))]
    {
        if player_query.iter().next().is_none() {
            spawn_player(
                &mut commands,
                #[cfg(feature = "graphics")]
                &mut meshes,
                #[cfg(feature = "graphics")]
                &mut materials,
                #[cfg(feature = "graphics")]
                &mut asset_server,
            );
        }
    }

    if ally_query.iter().next().is_none() {
        spawn_allies(
            &mut commands,
            #[cfg(feature = "graphics")]
            &mut meshes,
            #[cfg(feature = "graphics")]
            &mut materials,
            #[cfg(feature = "graphics")]
            &mut asset_server,
            1,
        );
    }
}
