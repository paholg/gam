use bevy::{
    app::{Plugin, Startup, Update},
    asset::{AssetServer, Assets},
    color::LinearRgba,
    ecs::{
        entity::Entity,
        query::Added,
        system::{Commands, Query, Res, ResMut, Resource},
        world::World,
    },
    math::primitives::Sphere,
    prelude::{Handle, Mesh, StandardMaterial, Vec2, Vec3, Vec4},
};
use bevy_hanabi::{
    Attribute, ColorOverLifetimeModifier, EffectAsset, ExprWriter, Gradient, LinearDragModifier,
    ParticleEffectBundle, SetAttributeModifier, SetPositionSphereModifier,
    SetVelocitySphereModifier, ShapeDimension, SizeOverLifetimeModifier, Spawner,
};
use bevy_kira_audio::AudioSource;
use engine::ability::{bullet::Bullet, gun::GunProps};
use iyes_progress::prelude::AssetsLoading;

use crate::{draw::ObjectGraphics, particles::ParticleEffectPool};

pub struct GunPlugin;

impl Plugin for GunPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, draw_bullet_system);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: ResMut<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    props: Res<GunProps>,
) {
    let effect = effects.add(bullet_effect(&props));
    let effect_pool = ParticleEffectBundle::new(effect).into();

    let shot_material = StandardMaterial {
        emissive: LinearRgba::rgb(0.0, 20_000.0, 20_000.0),
        ..Default::default()
    };

    let bullet = BulletAssets {
        mesh: meshes.add(Sphere::new(1.0)),
        material: materials.add(shot_material),
        collision_effect: effect_pool,
        spawn_sound: asset_server.load("third-party/audio/other/laserSmall_000.ogg"),
        despawn_sound: asset_server.load("third-party/audio/other/laserSmall_000.ogg"),
    };
    loading.add(&bullet.spawn_sound);
    loading.add(&bullet.despawn_sound);

    commands.add(|world: &mut World| world.insert_resource(bullet));
}

fn draw_bullet_system(
    mut commands: Commands,
    assets: Res<BulletAssets>,
    query: Query<Entity, Added<Bullet>>,
) {
    for entity in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        ecmds.insert(ObjectGraphics {
            material: assets.material.clone(),
            mesh: assets.mesh.clone(),
            ..Default::default()
        });
    }
}

#[derive(Resource)]
struct BulletAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    collision_effect: ParticleEffectPool,
    spawn_sound: Handle<AudioSource>,
    despawn_sound: Handle<AudioSource>,
}

fn bullet_effect(props: &GunProps) -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.5, Vec4::new(2.0, 2.0, 4.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 4.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 4.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let spawner = Spawner::once(250.0.into(), true);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(props.radius).expr(),
        dimension: ShapeDimension::Volume,
    };

    let vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(1.5).uniform(writer.lit(2.0)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.2).uniform(writer.lit(0.4)).expr(),
    };

    let age = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).uniform(writer.lit(0.2)).expr(),
    };

    let drag = LinearDragModifier {
        drag: writer.lit(5.0).expr(),
    };

    EffectAsset::new(vec![32768], spawner, writer.finish())
        .with_name("shot_particle_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .init(age)
        .update(drag)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}
