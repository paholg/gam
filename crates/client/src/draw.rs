use bevy::{
    core::FrameCount,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::{
        Added, Assets, BuildChildren, Bundle, Commands, Component, Entity, EventReader,
        GlobalTransform, Handle, InheritedVisibility, Mesh, Parent, PbrBundle, Plugin, Query, Res,
        ResMut, SpotLight, SpotLightBundle, StandardMaterial, Transform, Update, Vec2, Vec3,
        ViewVisibility, Visibility, With, Without,
    },
    scene::Scene,
};
use bevy_hanabi::EffectSpawner;
use bevy_kira_audio::{prelude::Volume, Audio, AudioControl};
use bevy_mod_raycast::prelude::RaycastMesh;
use bevy_rapier3d::prelude::{Collider, Velocity};
use engine::{
    ability::{
        bullet::Bullet,
        grenade::{Grenade, GrenadeKind},
        neutrino_ball::{NeutrinoBall, NeutrinoBallGravityField},
        seeker_rocket::SeekerRocket,
        transport::TransportBeam,
    },
    collision::TrackCollisions,
    death_callback::{Explosion, ExplosionKind},
    level::{Floor, InLevel, LevelProps, SHORT_WALL, WALL_HEIGHT},
    lifecycle::{DeathEvent, DEATH_Y},
    status_effect::TimeDilation,
    Ally, Enemy, Energy, FootOffset, Health, Kind, Player, To2d, To3d, UP,
};

use crate::{
    asset_handler::{AssetHandler, ExplosionAssets},
    bar::Bar,
    in_plane, Config,
};

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
                draw_neutrino_ball_system,
                draw_neutrino_ball_outline_system,
                draw_death_system,
                draw_wall_system,
                update_wall_system,
                draw_lights_system,
                draw_explosion_system,
                update_explosion_system,
                draw_transport_system,
                update_transport_system,
                draw_time_dilation_system,
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
                .spawn((
                    PbrBundle {
                        mesh,
                        material,
                        transform: in_plane().with_translation(Vec3::new(
                            0.0,
                            foot_offset.y + 0.01,
                            0.0,
                        )),
                        ..Default::default()
                    },
                    NotShadowCaster,
                    NotShadowReceiver,
                ))
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
                Bar::<Energy>::new(0.1, Vec2::new(0.3, 0.08)),
                in_plane(),
                GlobalTransform::default(),
            ));
        });
    }
}

fn draw_neutrino_ball_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<Entity, Added<NeutrinoBall>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert((InheritedVisibility::default(),));

        ecmds.with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: assets.neutrino_ball.material.clone(),
                    mesh: assets.neutrino_ball.mesh.clone(),
                    ..Default::default()
                },
                Transform::IDENTITY,
                GlobalTransform::default(),
            ));
        });
    }
}

fn draw_neutrino_ball_outline_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<
        (Entity, &FootOffset),
        (
            Added<NeutrinoBallGravityField>,
            Without<HasOutline>,
            Added<TrackCollisions>,
        ),
    >,
) {
    for (entity, foot_offset) in &query {
        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        let outline_entity = commands
            .spawn((
                PbrBundle {
                    mesh: assets.neutrino_ball.outline_mesh.clone(),
                    material: assets.neutrino_ball.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        commands
            .entity(entity)
            .insert(HasOutline)
            .push_children(&[outline_entity]);
    }
}

fn draw_player_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &FootOffset), Added<Player>>,
) {
    for (entity, foot_offset) in query.iter() {
        let outline = commands
            .spawn((
                PbrBundle {
                    mesh: assets.player.outline_mesh.clone(),
                    material: assets.player.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
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
            .spawn((
                PbrBundle {
                    mesh: assets.enemy.outline_mesh.clone(),
                    material: assets.enemy.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
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
            .spawn((
                PbrBundle {
                    mesh: assets.ally.outline_mesh.clone(),
                    material: assets.ally.outline_material.clone(),
                    transform: in_plane().with_translation(Vec3::new(
                        0.0,
                        foot_offset.y + 0.01,
                        0.0,
                    )),
                    ..Default::default()
                },
                NotShadowCaster,
                NotShadowReceiver,
            ))
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
            Kind::Other => None,
            Kind::Player => Some(&mut assets.player.despawn_effect),
            Kind::Enemy => Some(&mut assets.enemy.despawn_effect),
            Kind::Ally => Some(&mut assets.ally.despawn_effect),
            Kind::Bullet => Some(&mut assets.shot.collision_effect),
            Kind::FragGrenade
            | Kind::HealGrenade
            | Kind::SeekerRocket
            | Kind::NeutrinoBall
            | Kind::TransportBeam => None,
        };

        if let Some(effect) = effect {
            effect.trigger(&mut commands, death.transform, &mut effects, &frame);
        }

        let sound = match death.kind {
            Kind::Other | Kind::NeutrinoBall | Kind::TransportBeam => None,
            Kind::Bullet => Some(assets.shot.despawn_sound.clone()),
            Kind::Player
            | Kind::Enemy
            | Kind::Ally
            | Kind::FragGrenade
            | Kind::HealGrenade
            | Kind::SeekerRocket => Some(assets.player.despawn_sound.clone()),
        };

        if let Some(sound) = sound {
            audio
                .play(sound)
                .with_volume(Volume::Decibels(config.sound.effects_volume));
        }
    }
}

fn draw_time_dilation_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    query: Query<(&Transform, &FootOffset, &TimeDilation), Without<EffectSpawner>>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.time_dilation.fast_effect;

    for (transform, foot_offset, time_dilation) in query.iter() {
        // TODO: Add an effect for slow things.
        if time_dilation.factor() <= 1.0 {
            continue;
        }
        let mut effect_transform = *transform;
        effect_transform.translation.y += foot_offset.y;
        // TODO: Change the affect based on how big the effect is.
        effect.trigger(&mut commands, effect_transform, &mut effects, &frame);
    }
}

#[derive(Component, Copy, Clone)]
enum WallKind {
    Floor,
    Short,
    Standard,
    Tall,
}

impl WallKind {
    fn opaque(&self, assets: &AssetHandler) -> Handle<StandardMaterial> {
        match self {
            WallKind::Floor => assets.wall.floor.clone(),
            WallKind::Short => assets.wall.short_wall.clone(),
            WallKind::Standard => assets.wall.wall.clone(),
            WallKind::Tall => assets.wall.tall_wall.clone(),
        }
    }

    fn trans(&self, assets: &AssetHandler) -> Handle<StandardMaterial> {
        match self {
            WallKind::Floor => assets.wall.floor.clone(), // no trans floor
            WallKind::Short => assets.wall.short_wall_trans.clone(),
            WallKind::Standard => assets.wall.wall_trans.clone(),
            WallKind::Tall => assets.wall.tall_wall_trans.clone(),
        }
    }

    fn is_wall(&self) -> bool {
        match self {
            WallKind::Floor => false,
            WallKind::Short | WallKind::Standard | WallKind::Tall => true,
        }
    }
}

#[derive(Component)]
struct Wall;

fn draw_wall_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    query: Query<(Entity, &Floor), Added<Floor>>,
) {
    for (entity, floor) in &query {
        let kind = if floor.dim.y >= WALL_HEIGHT - DEATH_Y + 0.1 {
            WallKind::Tall
        } else if floor.dim.y >= WALL_HEIGHT - DEATH_Y - 0.1 {
            WallKind::Standard
        } else if floor.dim.y >= SHORT_WALL - DEATH_Y - 0.1 {
            WallKind::Short
        } else {
            WallKind::Floor
        };

        // First add InheritedVisibility to our entity to make bevy happy.
        commands
            .entity(entity)
            .insert(InheritedVisibility::default());

        // We want to chunk walls into a "floor" section and a "wall" section, so
        // we're only making the part above the floor transparent when it's
        // blocking a character.
        let props = if kind.is_wall() {
            let mut floor_scale = floor.dim;
            floor_scale.y = -DEATH_Y;

            let mut wall_scale = floor.dim;
            wall_scale.y = floor.dim.y + DEATH_Y;
            vec![
                (
                    Transform::from_scale(floor_scale).with_translation(Vec3::new(
                        0.0,
                        -wall_scale.y * 0.5,
                        0.0,
                    )),
                    WallKind::Floor,
                ),
                (
                    Transform::from_scale(wall_scale).with_translation(Vec3::new(
                        0.0,
                        floor_scale.y * 0.5,
                        0.0,
                    )),
                    kind,
                ),
            ]
        } else {
            vec![(Transform::from_scale(floor.dim), kind)]
        };

        let ids = props
            .into_iter()
            .map(|(transform, kind)| {
                let wall = commands
                    .spawn((
                        PbrBundle {
                            mesh: assets.wall.shape.clone(),
                            material: kind.opaque(&assets),
                            transform,
                            ..Default::default()
                        },
                        kind,
                    ))
                    .id();
                if kind.is_wall() {
                    commands
                        .entity(wall)
                        .insert((Wall, RaycastMesh::<()>::default()));
                }
                wall
            })
            .collect::<Vec<_>>();

        commands.entity(entity).push_children(&ids);
    }
}

fn update_wall_system(
    assets: Res<AssetHandler>,
    mut query: Query<
        (
            &mut Handle<StandardMaterial>,
            &Transform,
            &GlobalTransform,
            &WallKind,
        ),
        With<Wall>,
    >,
    healthbar_q: Query<(&GlobalTransform, &Bar<Health>)>,
) {
    const DELTA_Y: f32 = 1.3;

    struct BarInfo {
        loc: Vec2,
        size: Vec2,
    }

    let healthbars = healthbar_q
        .iter()
        .map(|(gt, bar)| BarInfo {
            loc: gt.translation().to_2d(),
            size: bar.size,
        })
        .collect::<Vec<_>>();
    // TODO: This is really inefficient.
    for (mut material, transform, global_transform, kind) in &mut query {
        let loc = global_transform.translation().to_2d();
        let shape = transform.scale.to_2d();

        let wall_left = loc.x - shape.x * 0.5;
        let wall_right = loc.x + shape.x * 0.5;
        let wall_top = loc.y + shape.y * 0.5;

        if healthbars.iter().any(|hb| {
            let hb_left = hb.loc.x - hb.size.x * 0.5;
            let hb_right = hb.loc.x + hb.size.x * 0.5;
            let hb_bottom = hb.loc.y + hb.size.y * 0.5;

            // Check if this bar is being blocked visually by this wall.
            hb_left < wall_right
                && hb_right > wall_left
                && hb_bottom > wall_top
                && hb_bottom < wall_top + DELTA_Y
        }) {
            *material = kind.trans(&assets);
        } else {
            *material = kind.opaque(&assets);
        }
    }
}

fn draw_lights_system(mut commands: Commands, level: Res<LevelProps>, query: Query<&SpotLight>) {
    if query.iter().next().is_some() {
        return;
    }
    let altitude = 10.0;
    let spacing = 15.0;

    let nx = (level.x / spacing).ceil().max(1.0) as usize;
    let nz = (level.z / spacing).ceil().max(1.0) as usize;

    let offset = |n| {
        if n % 2 == 0 {
            let offset = ((n as f32) * 0.5 - 1.0) * spacing + spacing * 0.5;
            -offset
        } else {
            let offset = (n as f32 - 1.0) * 0.5 * spacing;
            -offset
        }
    };

    let xoffset = offset(nx);
    let zoffset = offset(nz);

    for x in 0..nx {
        for z in 0..nz {
            let x = (x as f32) * spacing + xoffset;
            let z = (z as f32) * spacing + zoffset;

            // Offset the light a bit, for more interesting shadows.
            let t = Transform::from_xyz(x - spacing * 0.5, altitude, z - spacing)
                .looking_at(Vec3::new(x, 0.0, z), UP);

            commands.spawn((
                SpotLightBundle {
                    spot_light: SpotLight {
                        shadows_enabled: true,
                        range: 30.0,
                        intensity: 4000.0,
                        outer_angle: std::f32::consts::FRAC_PI_3,
                        inner_angle: 0.0,
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

#[derive(Component)]
pub struct ExplosionGraphics;

fn explosion_assets(assets: &AssetHandler, kind: ExplosionKind) -> &ExplosionAssets {
    match kind {
        ExplosionKind::FragGrenade => &assets.frag_grenade.explosion,
        ExplosionKind::HealGrenade => &assets.heal_grenade.explosion,
        ExplosionKind::SeekerRocket => &assets.seeker_rocket.explosion,
    }
}

pub fn draw_explosion_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Explosion, &Collider), Added<Explosion>>,
) {
    for (entity, explosion, collider) in &query {
        let explosion_assets = explosion_assets(&assets, explosion.kind);
        let radius = collider.as_ball().unwrap().radius();

        // Clone the material because we're going to mutate it. Probably we
        // could do this better.
        let material = materials.get(&explosion_assets.material).unwrap().clone();

        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        commands.entity(entity).with_children(|builder| {
            builder.spawn((
                ObjectGraphics {
                    material: materials.add(material),
                    mesh: explosion_assets.mesh.clone(),
                    ..Default::default()
                },
                Transform::from_scale(Vec3::splat(radius)),
                GlobalTransform::default(),
                ExplosionGraphics,
            ));
        });
    }
}

pub fn update_explosion_system(
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&Parent, &mut Transform, &Handle<StandardMaterial>), With<ExplosionGraphics>>,
    parent_q: Query<(&Explosion, &Collider)>,
) {
    for (parent, mut transform, material) in &mut query {
        let Ok((explosion, collider)) = parent_q.get(parent.get()) else {
            tracing::warn!("ExplosionGraphics missing parent");
            continue;
        };
        let radius = collider.as_ball().unwrap().radius();

        let min_radius = explosion.min_radius;
        let max_radius = explosion.max_radius;

        let frac = (radius - min_radius) / (max_radius - min_radius);
        let explosion_assets = explosion_assets(&assets, explosion.kind);

        let color = explosion_assets.gradient.get(frac);
        materials.get_mut(material).unwrap().emissive = color;

        transform.scale = Vec3::splat(radius);
    }
}

#[derive(Component)]
pub struct TransportSenderGraphics {
    receiver: Entity,
}

#[derive(Component)]
pub struct TransportReceiverGraphics;

pub fn draw_transport_system(
    mut commands: Commands,
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &TransportBeam, &Transform), Added<TransportBeam>>,
) {
    for (entity, beam, transform) in &query {
        // Clone the material because we're going to mutate it. Probably we
        // could do this better.
        let mat = materials.get(&assets.transport.material).unwrap().clone();
        let material = materials.add(mat);

        commands
            .entity(entity)
            .insert(InheritedVisibility::default());
        commands.entity(entity).with_children(|builder| {
            let receiver = builder
                .spawn((
                    ObjectGraphics {
                        material: material.clone(),
                        mesh: assets.transport.mesh.clone(),
                        ..Default::default()
                    },
                    Transform::from_scale(Vec3::new(beam.radius, 0.0, beam.radius))
                        .with_translation(beam.destination.to_3d(0.0) - transform.translation),
                    GlobalTransform::default(),
                    TransportReceiverGraphics,
                    NotShadowReceiver,
                ))
                .id();

            builder.spawn((
                ObjectGraphics {
                    material,
                    mesh: assets.transport.mesh.clone(),
                    ..Default::default()
                },
                Transform::from_scale(Vec3::new(beam.radius, 0.0, beam.radius))
                    .with_translation(Vec3::new(0.0, beam.height, 0.0)),
                GlobalTransform::default(),
                TransportSenderGraphics { receiver },
                NotShadowReceiver,
            ));
        });
    }
}

pub fn update_transport_system(
    assets: Res<AssetHandler>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sender_q: Query<(
        &Parent,
        &mut Transform,
        &Handle<StandardMaterial>,
        &TransportSenderGraphics,
    )>,
    mut receiver_q: Query<
        &mut Transform,
        (
            With<TransportReceiverGraphics>,
            Without<TransportSenderGraphics>,
        ),
    >,
    parent_q: Query<
        (&Transform, &TransportBeam),
        (
            Without<TransportSenderGraphics>,
            Without<TransportReceiverGraphics>,
        ),
    >,
) {
    for (parent, mut transform, material, sender) in &mut sender_q {
        let Ok((parent_transform, beam)) = parent_q.get(parent.get()) else {
            tracing::warn!("TransportSenderGraphics missing parent");
            continue;
        };

        let frac = beam.activates_in / beam.delay;

        let color = assets.transport.gradient.get(frac);
        materials.get_mut(material).unwrap().base_color = color;
        transform.scale.y = frac * beam.height;
        transform.translation.y = beam.height - (frac * beam.height * 0.5);

        let Ok(mut receiver_transform) = receiver_q.get_mut(sender.receiver) else {
            tracing::warn!("Transport sender with no receiver");
            continue;
        };
        receiver_transform.scale.y = frac * beam.height;
        receiver_transform.translation =
            beam.destination.to_3d(frac * beam.height * 0.5) - parent_transform.translation;
    }
}
