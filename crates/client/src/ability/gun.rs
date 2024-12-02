use bevy::app::Plugin;
use bevy::app::Startup;
use bevy::app::Update;
use bevy::asset::AssetServer;
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::color::LinearRgba;
use bevy::core::FrameCount;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::Added;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::system::ResMut;
use bevy::ecs::system::Resource;
use bevy::ecs::system::SystemId;
use bevy::ecs::world::World;
use bevy::math::primitives::Sphere;
use bevy::prelude::Handle;
use bevy::prelude::In;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy::prelude::Vec4;
use bevy::prelude::Without;
use bevy_hanabi::Attribute;
use bevy_hanabi::ColorOverLifetimeModifier;
use bevy_hanabi::EffectAsset;
use bevy_hanabi::EffectInitializers;
use bevy_hanabi::ExprWriter;
use bevy_hanabi::Gradient;
use bevy_hanabi::LinearDragModifier;
use bevy_hanabi::ParticleEffectBundle;
use bevy_hanabi::SetAttributeModifier;
use bevy_hanabi::SetPositionSphereModifier;
use bevy_hanabi::SetVelocitySphereModifier;
use bevy_hanabi::ShapeDimension;
use bevy_hanabi::SizeOverLifetimeModifier;
use bevy_hanabi::Spawner;
use bevy_kira_audio::prelude::Volume;
use bevy_kira_audio::Audio;
use bevy_kira_audio::AudioControl;
use bevy_kira_audio::AudioSource;
use engine::ability::bullet::Bullet;
use engine::ability::gun::GunKind;
use engine::ability::gun::GunProps;
use engine::ability::gun::StandardGun;
use engine::lifecycle::ClientDeathCallback;
use iyes_progress::prelude::AssetsLoading;

use crate::draw::ObjectGraphics;
use crate::particles::ParticleEffectPool;
use crate::Config;

pub struct GunPlugin;

#[derive(Resource)]
struct BulletAssets {
    mesh: Handle<Mesh>,
    standard_material: Handle<StandardMaterial>,
    fire_material: Handle<StandardMaterial>,
    cold_material: Handle<StandardMaterial>,
    collision_effect: ParticleEffectPool,
    spawn_sound: Handle<AudioSource>,
    despawn_sound: Handle<AudioSource>,
}

#[derive(Resource)]
struct GunDeathCallback {
    system: SystemId<Entity>,
}

impl Plugin for GunPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let callback = GunDeathCallback {
            system: app.register_system(bullet_death_system),
        };
        app.insert_resource(callback)
            .add_systems(Startup, setup)
            .add_systems(Update, draw_bullet);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut effects: ResMut<Assets<EffectAsset>>,
    asset_server: ResMut<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    standard_props: Res<GunProps<StandardGun>>,
    // fire_props: Res<GunProps<FireGun>>,
    // cold_props: Res<GunProps<ColdGun>>,
) {
    let effect = effects.add(bullet_effect(&standard_props));
    let effect_pool = ParticleEffectBundle::new(effect).into();

    let standard_material = StandardMaterial {
        base_color: Color::linear_rgb(0.2, 0.2, 0.2),
        ..Default::default()
    };

    let fire_material = StandardMaterial {
        emissive: LinearRgba::rgb(20.0, 10.0, 0.0),
        ..Default::default()
    };

    let cold_material = StandardMaterial {
        emissive: LinearRgba::rgb(0.0, 10.0, 20.0),
        ..Default::default()
    };

    let bullet = BulletAssets {
        mesh: meshes.add(Sphere::new(1.0)),
        standard_material: materials.add(standard_material),
        fire_material: materials.add(fire_material),
        cold_material: materials.add(cold_material),
        collision_effect: effect_pool,
        spawn_sound: asset_server.load("third-party/audio/other/laserSmall_000.ogg"),
        despawn_sound: asset_server.load("third-party/audio/other/laserSmall_000.ogg"),
    };
    loading.add(&bullet.spawn_sound);
    loading.add(&bullet.despawn_sound);

    commands.add(|world: &mut World| world.insert_resource(bullet));
}

fn bullet_death_system(
    In(entity): In<Entity>,
    query: Query<&Transform, Without<EffectInitializers>>,
    mut commands: Commands,
    mut assets: ResMut<BulletAssets>,
    audio: Res<Audio>,
    config: Res<Config>,
    mut effects: Query<(&mut Transform, &mut EffectInitializers)>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.collision_effect;
    let transform = *query.get(entity).unwrap();
    effect.trigger(&mut commands, transform, &mut effects, &frame);

    let sound = assets.despawn_sound.clone_weak();
    audio
        .play(sound)
        .with_volume(Volume::Decibels(config.sound.effects_volume));
}

fn draw_bullet(
    mut commands: Commands,
    assets: Res<BulletAssets>,
    death_callback: Res<GunDeathCallback>,
    query: Query<(Entity, &Bullet), Added<Bullet>>,
) {
    for (entity, bullet) in query.iter() {
        let Some(mut ecmds) = commands.get_entity(entity) else {
            continue;
        };
        let material = if bullet.heat > 0.0 {
            assets.fire_material.clone_weak()
        } else if bullet.heat < 0.0 {
            assets.cold_material.clone_weak()
        } else {
            assets.standard_material.clone_weak()
        };
        ecmds.insert((
            ClientDeathCallback::new(death_callback.system),
            ObjectGraphics {
                material,
                mesh: assets.mesh.clone_weak(),
                ..Default::default()
            },
        ));
    }
}

fn bullet_effect<G: GunKind>(props: &GunProps<G>) -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(0.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.5, Vec4::new(2.0, 2.0, 4.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 4.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 4.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec3::splat(0.1));
    size_gradient1.add_key(0.3, Vec3::splat(0.1));
    size_gradient1.add_key(1.0, Vec3::splat(0.0));

    let spawner = Spawner::once(250.0.into(), true);
    let writer = ExprWriter::new();

    let pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(props.bullet.radius).expr(),
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

    EffectAsset::new(32768, spawner, writer.finish())
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
