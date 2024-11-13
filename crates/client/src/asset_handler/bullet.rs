use bevy::color::LinearRgba;
use bevy::math::primitives::Sphere;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Vec2;
use bevy::prelude::Vec3;
use bevy::prelude::Vec4;
use bevy_hanabi::Attribute;
use bevy_hanabi::ColorOverLifetimeModifier;
use bevy_hanabi::EffectAsset;
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
use bevy_kira_audio::AudioSource;
use engine::ability::properties::GunProps;

use super::Builder;
use crate::particles::ParticleEffectPool;

pub struct BulletAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub collision_effect: ParticleEffectPool,
    pub spawn_sound: Handle<AudioSource>,
    pub despawn_sound: Handle<AudioSource>,
}

impl BulletAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let effect = builder.effects.add(bullet_effect(&builder.props.gun));
        let effect_pool = ParticleEffectBundle::new(effect).into();

        let shot_material = StandardMaterial {
            emissive: LinearRgba::rgb(0.0, 20_000.0, 20_000.0),
            ..Default::default()
        };

        let bullet = BulletAssets {
            mesh: builder.meshes.add(Sphere::new(1.0)),
            material: builder.materials.add(shot_material),
            collision_effect: effect_pool,
            spawn_sound: builder
                .asset_server
                .load("third-party/audio/other/laserSmall_000.ogg"),
            despawn_sound: builder
                .asset_server
                .load("third-party/audio/other/laserSmall_000.ogg"),
        };
        builder.loading.add(&bullet.spawn_sound);
        builder.loading.add(&bullet.despawn_sound);
        bullet
    }
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
