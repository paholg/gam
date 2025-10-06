use std::marker::PhantomData;

use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    audio::AudioSource,
    color::{
        palettes::css::{GREEN, LIGHT_CYAN, RED},
        Alpha, LinearRgba,
    },
    diagnostic::FrameCount,
    ecs::{component::Component, system::SystemId},
    light::{NotShadowCaster, NotShadowReceiver},
    math::Vec4,
    mesh::Mesh,
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::{
        Added, Commands, Entity, In, InheritedVisibility, Mesh3d, Query, Res, ResMut, Resource,
        Transform, Vec3, Without,
    },
    scene::SceneRoot,
};
use bevy_hanabi::{
    Attribute, ColorOverLifetimeModifier, EffectAsset, EffectSpawner, ExprWriter, Gradient,
    LinearDragModifier, SetAttributeModifier, SetPositionSphereModifier, SetVelocitySphereModifier,
    ShapeDimension, SizeOverLifetimeModifier, SpawnerSettings,
};
use engine::{
    lifecycle::ClientDeathCallback, Ally, Enemy, Energy, FootOffset, Health, Player, PLAYER_R,
};

use crate::{
    aim::BlocksSight, audio::play_sound_effect, bar::Bar, in_plane, particles::ParticleEffectPool,
    shapes::HollowPolygon, Config,
};

pub struct CharacterPlugin;

#[derive(Resource)]
struct CharacterDeathCallbacks {
    player: SystemId<In<Entity>>,
    enemy: SystemId<In<Entity>>,
    ally: SystemId<In<Entity>>,
}

#[derive(Component)]
pub struct CharacterMarker;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let callbacks = CharacterDeathCallbacks {
            player: app.register_system(player_death_system),
            enemy: app.register_system(enemy_death_system),
            ally: app.register_system(ally_death_system),
        };

        app.insert_resource(callbacks)
            .add_systems(Startup, create_assets)
            .add_systems(
                Update,
                (draw_player_system, draw_enemy_system, draw_ally_system),
            );
    }
}

// asset_server: &AssetServer,
// meshes: &mut Assets<Mesh>,
// materials: &mut Assets<StandardMaterial>,
// effects: &mut Assets<EffectAsset>,
fn create_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    commands.insert_resource(CharacterAssets::player(
        &asset_server,
        &mut meshes,
        &mut materials,
        &mut effects,
    ));
    commands.insert_resource(CharacterAssets::enemy(
        &asset_server,
        &mut meshes,
        &mut materials,
        &mut effects,
    ));
    commands.insert_resource(CharacterAssets::ally(
        &asset_server,
        &mut meshes,
        &mut materials,
        &mut effects,
    ));
}

fn player_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectSpawner>>,
    mut commands: Commands,
    mut assets: ResMut<CharacterAssets<Player>>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.despawn_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.despawn_sound.clone();
    play_sound_effect(&mut commands, &config, sound, transform);
}

fn enemy_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectSpawner>>,
    mut commands: Commands,
    mut assets: ResMut<CharacterAssets<Enemy>>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.despawn_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.despawn_sound.clone();
    play_sound_effect(&mut commands, &config, sound, transform);
}

fn ally_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectSpawner>>,
    mut commands: Commands,
    mut assets: ResMut<CharacterAssets<Ally>>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.despawn_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.despawn_sound.clone();
    play_sound_effect(&mut commands, &config, sound, transform);
}

fn draw_player_system(
    mut commands: Commands,
    assets: Res<CharacterAssets<Player>>,
    callbacks: Res<CharacterDeathCallbacks>,
    query: Query<(Entity, &FootOffset), Added<Player>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert((
                InheritedVisibility::VISIBLE,
                CharacterMarker,
                ClientDeathCallback::new(callbacks.player),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Mesh3d::from(assets.outline_mesh.clone()),
                    MeshMaterial3d::from(assets.outline_material.clone()),
                    in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn((
                    assets.scene.clone(),
                    Transform::from_translation(foot_offset.to_vec()),
                    BlocksSight,
                    Bar::<Health>::default(),
                    Bar::<Energy>::default(),
                ));
            });
    }
}

fn draw_enemy_system(
    mut commands: Commands,
    assets: Res<CharacterAssets<Enemy>>,
    callbacks: Res<CharacterDeathCallbacks>,
    query: Query<(Entity, &FootOffset), Added<Enemy>>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert((
                InheritedVisibility::VISIBLE,
                CharacterMarker,
                ClientDeathCallback::new(callbacks.enemy),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Mesh3d::from(assets.outline_mesh.clone()),
                    MeshMaterial3d::from(assets.outline_material.clone()),
                    in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn((
                    assets.scene.clone(),
                    Transform::from_translation(foot_offset.to_vec()),
                    BlocksSight,
                    Bar::<Health>::default(),
                    Bar::<Energy>::default(),
                ));
            });
    }
}

fn draw_ally_system(
    mut commands: Commands,
    assets: Res<CharacterAssets<Ally>>,
    callbacks: Res<CharacterDeathCallbacks>,
    query: Query<(Entity, &FootOffset), (Added<Ally>, Without<Player>)>,
) {
    for (entity, foot_offset) in query.iter() {
        commands
            .entity(entity)
            .insert((
                InheritedVisibility::default(),
                CharacterMarker,
                ClientDeathCallback::new(callbacks.ally),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Mesh3d::from(assets.outline_mesh.clone()),
                    MeshMaterial3d::from(assets.outline_material.clone()),
                    in_plane().with_translation(Vec3::new(0.0, foot_offset.y, 0.0)),
                    NotShadowCaster,
                    NotShadowReceiver,
                ));
                builder.spawn((
                    assets.scene.clone(),
                    Transform::from_translation(foot_offset.to_vec()),
                    BlocksSight,
                    Bar::<Health>::default(),
                    Bar::<Energy>::default(),
                ));
            });
    }
}

#[derive(Resource)]
struct CharacterAssets<T> {
    scene: SceneRoot,
    outline_mesh: Handle<Mesh>,
    outline_material: Handle<StandardMaterial>,
    despawn_sound: Handle<AudioSource>,
    despawn_effect: ParticleEffectPool,
    _marker: PhantomData<T>,
}

impl<T> CharacterAssets<T> {
    fn outline(
        color: LinearRgba,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        let mesh = meshes.add(HollowPolygon {
            radius: PLAYER_R,
            thickness: 0.04,
            vertices: 30,
        });

        const OUTLINE_ALPHA: f32 = 0.5;
        let material = materials.add(StandardMaterial {
            unlit: true,
            base_color: color.with_alpha(OUTLINE_ALPHA).into(),
            // TODO: Make actually emissive???
            emissive: color.with_alpha(OUTLINE_ALPHA),
            ..Default::default()
        });

        (mesh, material)
    }

    fn character(
        asset_server: &AssetServer,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        effects: &mut Assets<EffectAsset>,
        color: LinearRgba,
        model_path: &'static str,
    ) -> Self {
        // let model = builder.asset_server.load("models/temp/robot1.glb#Scene0");
        let model = asset_server.load(model_path);
        let despawn_sound = asset_server.load("third-party/audio/other/explosionCrunch_000.ogg");

        let despawn_effect = effects.add(death_effect()).into();

        let (outline_mesh, outline_material) = Self::outline(color, meshes, materials);
        CharacterAssets {
            scene: SceneRoot(model),
            outline_mesh,
            outline_material,
            despawn_sound,
            despawn_effect,
            _marker: PhantomData,
        }
    }
}

impl CharacterAssets<Player> {
    fn player(
        asset_server: &AssetServer,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        effects: &mut Assets<EffectAsset>,
    ) -> Self {
        Self::character(
            asset_server,
            meshes,
            materials,
            effects,
            GREEN.into(),
            "models/temp/robot1.glb#Scene0",
        )
    }
}

impl CharacterAssets<Ally> {
    fn ally(
        asset_server: &AssetServer,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        effects: &mut Assets<EffectAsset>,
    ) -> Self {
        Self::character(
            asset_server,
            meshes,
            materials,
            effects,
            LIGHT_CYAN.into(),
            "models/temp/robot1.glb#Scene0",
        )
    }
}

impl CharacterAssets<Enemy> {
    fn enemy(
        asset_server: &AssetServer,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        effects: &mut Assets<EffectAsset>,
    ) -> Self {
        Self::character(
            asset_server,
            meshes,
            materials,
            effects,
            RED.into(),
            "models/temp/snowman.glb#Scene0",
        )
    }
}

fn death_effect() -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 4.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec3::splat(0.05));
    size_gradient1.add_key(0.3, Vec3::splat(0.07));
    size_gradient1.add_key(1.0, Vec3::splat(0.0));

    let spawner = SpawnerSettings::once(500.0.into());
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(PLAYER_R).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(1.5).uniform(writer.lit(2.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.4).uniform(writer.lit(0.6)).expr(),
    };

    let age = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).uniform(writer.lit(0.2)).expr(),
    };

    let drag = LinearDragModifier {
        drag: writer.lit(5.0).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("death_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .init(age)
        .update(drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
            ..Default::default()
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}
