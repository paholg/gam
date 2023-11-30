use bevy::{
    core::FrameCount,
    prelude::{
        shape, Added, Assets, BuildChildren, Bundle, Color, Commands, Component, Entity,
        EventReader, GlobalTransform, Handle, InheritedVisibility, Mesh, Parent, PbrBundle, Plugin,
        Query, Res, ResMut, SpotLight, SpotLightBundle, StandardMaterial, Transform, Update, Vec2,
        Vec3, ViewVisibility, Visibility, With, Without,
    },
    scene::Scene,
};
use bevy_hanabi::EffectSpawner;
use bevy_kira_audio::{prelude::Volume, Audio, AudioControl};
use bevy_mod_raycast::prelude::RaycastMesh;
use bevy_rapier3d::prelude::Velocity;
use engine::{
    ability::{
        bullet::Bullet,
        grenade::{Grenade, GrenadeKind},
        seeker_rocket::SeekerRocket,
        HyperSprinting,
    },
    level::{Floor, InLevel, LevelProps, SHORT_WALL, WALL_HEIGHT},
    lifecycle::{DeathEvent, DEATH_Y},
    Ally, Enemy, Energy, FootOffset, Health, Kind, Player, UP,
};

use crate::{asset_handler::AssetHandler, bar::Bar, in_plane, Config};

/// A plugin for spawning graphics for newly-created entities.
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                raycast_scene_system,
                draw_player_system,
                draw_enemy_system,
                draw_ally_system,
                draw_shot_system,
                draw_grenade_system,
                draw_grenade_outline_system,
                draw_seeker_rocket_system,
                draw_death_system,
                draw_hyper_sprint_system,
                draw_wall_system,
                draw_lights_system,
            ),
        );
    }
}

/// A Component to be added to entities with `Scene`s, that we want to act as
/// `RaycastMesh<()>`s.
#[derive(Component, Default)]
struct RaycastScene;

#[derive(Bundle, Default)]
struct ObjectGraphics {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

#[derive(Bundle, Default)]
struct CharacterGraphics {
    healthbar: Bar<Health>,
    energybar: Bar<Energy>,
    scene: Handle<Scene>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    raycast: RaycastScene,
    transform: Transform,
    global_transform: GlobalTransform,
}

fn raycast_scene_system(
    mut commands: Commands,
    q_meshes: Query<(Entity, &Parent), Added<Handle<Mesh>>>,
    q_children: Query<&Parent>,
    q_parents: Query<&Handle<Scene>, With<RaycastScene>>,
) {
    // A `Scene` has meshes as its grandchildren, so we need a silly bit of
    // indirection to tell if we should add our `RaycastMesh`.
    for (entity, parent) in q_meshes.iter() {
        let Ok(parent) = q_children.get(parent.get()) else {
            continue;
        };
        let Ok(grandparent) = q_children.get(parent.get()) else {
            continue;
        };
        if q_parents.get(grandparent.get()).is_ok() {
            commands.entity(entity).insert(RaycastMesh::<()>::default());
        }
    }
}

fn draw_shot_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<Bullet>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(ObjectGraphics {
            material: assets.shot.material.clone(),
            mesh: assets.shot.mesh.clone(),
            ..Default::default()
        });
    }
}

fn draw_grenade_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Grenade), Added<Grenade>>,
) {
    for (entity, grenade) in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        let (mesh, material) = match grenade.kind {
            GrenadeKind::Frag => (
                assets.frag_grenade.mesh.clone(),
                assets.frag_grenade.material.clone(),
            ),
            GrenadeKind::Heal => (
                assets.heal_grenade.mesh.clone(),
                assets.heal_grenade.material.clone(),
            ),
        };
        ecmds.insert(ObjectGraphics {
            material,
            mesh,
            ..Default::default()
        });
    }
}

#[derive(Component)]
struct HasOutline;

fn draw_grenade_outline_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Grenade, &Velocity, &FootOffset), Without<HasOutline>>,
) {
    for (entity, grenade, velocity, foot_offset) in &query {
        if velocity.linvel.length_squared() < 0.1 * 0.1 {
            let (mesh, material) = match grenade.kind {
                GrenadeKind::Frag => (
                    assets.frag_grenade.outline_mesh.clone(),
                    assets.frag_grenade.outline_material.clone(),
                ),
                GrenadeKind::Heal => (
                    assets.heal_grenade.outline_mesh.clone(),
                    assets.heal_grenade.outline_material.clone(),
                ),
            };
            let outline_entity = commands
                .spawn(PbrBundle {
                    mesh,
                    material,
                    transform: in_plane().with_translation(foot_offset.to_vec()),
                    ..Default::default()
                })
                .id();
            commands
                .entity(entity)
                .insert(HasOutline)
                .push_children(&[outline_entity]);
        }
    }
}

fn draw_seeker_rocket_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<SeekerRocket>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: assets.seeker_rocket.material.clone(),
                    mesh: assets.seeker_rocket.mesh.clone(),
                    ..Default::default()
                },
                Bar::<Energy>::new(0.3, Vec2::new(1.2, 0.3)),
                in_plane(),
                GlobalTransform::default(),
            ));
        });
    }
}

fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), Added<Player>>,
) {
    for (entity, foot_offset) in query.iter() {
        let outline = commands
            .spawn(PbrBundle {
                mesh: assets.player.outline_mesh.clone(),
                material: assets.player.outline_material.clone(),
                transform: in_plane().with_translation(Vec3::new(0.0, foot_offset.y + 0.01, 0.0)),
                ..Default::default()
            })
            .id();
        let graphics = commands
            .spawn(CharacterGraphics {
                scene: assets.player.scene.clone(),
                transform: Transform::from_translation(foot_offset.to_vec()),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(InheritedVisibility::default())
            .push_children(&[outline, graphics]);
    }
}

fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), Added<Enemy>>,
) {
    for (entity, foot_offset) in query.iter() {
        let outline = commands
            .spawn(PbrBundle {
                mesh: assets.enemy.outline_mesh.clone(),
                material: assets.enemy.outline_material.clone(),
                transform: in_plane().with_translation(Vec3::new(0.0, foot_offset.y + 0.01, 0.0)),
                ..Default::default()
            })
            .id();
        let graphics = commands
            .spawn(CharacterGraphics {
                scene: assets.enemy.scene.clone(),
                transform: Transform::from_translation(foot_offset.to_vec()),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(InheritedVisibility::default())
            .push_children(&[outline, graphics]);
    }
}

fn draw_ally_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), (Added<Ally>, Without<Player>)>,
) {
    for (entity, foot_offset) in query.iter() {
        let outline = commands
            .spawn(PbrBundle {
                mesh: assets.ally.outline_mesh.clone(),
                material: assets.ally.outline_material.clone(),
                transform: in_plane().with_translation(Vec3::new(0.0, foot_offset.y + 0.01, 0.0)),
                ..Default::default()
            })
            .id();
        let graphics = commands
            .spawn(CharacterGraphics {
                scene: assets.ally.scene.clone(),
                transform: Transform::from_translation(foot_offset.to_vec()),
                ..Default::default()
            })
            .id();
        commands
            .entity(entity)
            .insert(InheritedVisibility::default())
            .push_children(&[outline, graphics]);
    }
}

fn draw_death_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    mut event_reader: EventReader<DeathEvent>,
    frame: Res<FrameCount>,
) {
    // TODO: Reset
    for death in event_reader.read() {
        let effect = match death.kind {
            Kind::Other => continue,
            Kind::Player => &mut assets.player.despawn_effect,
            Kind::Enemy => &mut assets.enemy.despawn_effect,
            Kind::Ally => &mut assets.ally.despawn_effect,
            Kind::Bullet => &mut assets.shot.collision_effect,
            Kind::FragGrenade => &mut assets.frag_grenade.explosion_effect,
            Kind::HealGrenade => &mut assets.heal_grenade.explosion_effect,
            Kind::SeekerRocket => &mut assets.seeker_rocket.explosion_effect,
        };

        effect.trigger(&mut commands, death.transform, &mut effects, &frame);

        let sound = match death.kind {
            Kind::Bullet => assets.shot.despawn_sound.clone(),
            Kind::Other => continue,
            Kind::Player
            | Kind::Enemy
            | Kind::Ally
            | Kind::FragGrenade
            | Kind::HealGrenade
            | Kind::SeekerRocket => assets.player.despawn_sound.clone(),
        };

        audio
            .play(sound)
            .with_volume(Volume::Decibels(config.sound.effects_volume));
    }
}

fn draw_hyper_sprint_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    query: Query<(&Transform, &FootOffset), (With<HyperSprinting>, Without<EffectSpawner>)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.hyper_sprint.effect;

    for (sprint_transform, foot_offset) in query.iter() {
        let mut transform = sprint_transform.clone();
        transform.translation.y += foot_offset.y;
        effect.trigger(&mut commands, transform, &mut effects, &frame);
    }
}

fn draw_wall_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Floor), Added<Floor>>,
) {
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::rgba(0.6, 0.8, 0.2, 0.8),
        ..Default::default()
    });
    let short_wall_mat = materials.add(Color::ALICE_BLUE.into());
    let wall_mat = materials.add(Color::AQUAMARINE.into());
    let tall_wall_mat = materials.add(Color::RED.into());

    for (entity, floor) in &query {
        let shape = shape::Box {
            min_x: -floor.dim.x * 0.5,
            max_x: floor.dim.x * 0.5,
            min_y: -floor.dim.y * 0.5,
            max_y: floor.dim.y * 0.5,
            min_z: -floor.dim.z * 0.5,
            max_z: floor.dim.z * 0.5,
        };

        let material = if floor.dim.y >= WALL_HEIGHT - DEATH_Y + 0.1 {
            tall_wall_mat.clone()
        } else if floor.dim.y >= WALL_HEIGHT - DEATH_Y - 0.1 {
            wall_mat.clone()
        } else if floor.dim.y >= SHORT_WALL - DEATH_Y - 0.1 {
            short_wall_mat.clone()
        } else {
            floor_mat.clone()
        };

        // First add InheritedVisibility to our entity to make bevy happy.
        commands
            .entity(entity)
            .insert(InheritedVisibility::default());

        let wall = commands
            .spawn((
                PbrBundle {
                    mesh: meshes.add(shape.into()),
                    material,
                    ..Default::default()
                },
                RaycastMesh::<()>::default(),
            ))
            .id();
        commands.entity(entity).push_children(&[wall]);
    }
}

fn draw_lights_system(mut commands: Commands, level: Res<LevelProps>, query: Query<&SpotLight>) {
    if query.iter().next().is_some() {
        return;
    }
    let step_size = 20.0;
    let xmin = (-level.x * 0.5 / step_size).round() as i32;
    let xmax = -xmin;
    let zmin = (-level.z * 0.5 / step_size).round() as i32;
    let zmax = -zmin;

    for x in xmin..=xmax {
        for z in zmin..=zmax {
            let x = x as f32 * step_size;
            let z = z as f32 * step_size;

            // Offset the light a bit, for more interesting shadows.
            let t =
                Transform::from_xyz(x - step_size, 20.0, z).looking_at(Vec3::new(x, 0.0, z), UP);

            commands.spawn((
                SpotLightBundle {
                    spot_light: SpotLight {
                        shadows_enabled: true,
                        range: 40.0,
                        intensity: 8000.0,
                        ..Default::default()
                    },
                    transform: t,
                    ..Default::default()
                },
                InLevel,
            ));
        }
    }
}
