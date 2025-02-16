use bevy::color::palettes::css::GREEN;
use bevy::color::palettes::css::LIGHT_CYAN;
use bevy::color::palettes::css::RED;
use bevy::color::Alpha;
use bevy::color::LinearRgba;
use bevy::prelude::Handle;
use bevy::prelude::Mesh;
use bevy::prelude::StandardMaterial;
use bevy::prelude::Vec3;
use bevy::prelude::Vec4;
use bevy::scene::SceneRoot;
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
use engine::PLAYER_R;

use super::Builder;
use crate::particles::ParticleEffectPool;
use crate::shapes::HollowPolygon;

pub struct CharacterAssets {
    pub scene: SceneRoot,
    pub outline_mesh: Handle<Mesh>,
    pub outline_material: Handle<StandardMaterial>,
    pub despawn_sound: Handle<AudioSource>,
    pub despawn_effect: ParticleEffectPool,
}

impl CharacterAssets {
    fn outline(
        builder: &mut Builder,
        color: LinearRgba,
    ) -> (Handle<Mesh>, Handle<StandardMaterial>) {
        let mesh = builder.meshes.add(HollowPolygon {
            radius: PLAYER_R,
            thickness: 0.04,
            vertices: 30,
        });

        const OUTLINE_ALPHA: f32 = 0.5;
        let material = builder.materials.add(StandardMaterial {
            unlit: true,
            base_color: color.with_alpha(OUTLINE_ALPHA).into(),
            // TODO: Make actually emissive???
            emissive: color.with_alpha(OUTLINE_ALPHA),
            ..Default::default()
        });

        (mesh, material)
    }

    fn character(builder: &mut Builder, color: LinearRgba, model_path: &'static str) -> Self {
        // let model = builder.asset_server.load("models/temp/robot1.glb#Scene0");
        let model = builder.asset_server.load(model_path);
        let despawn_sound = builder
            .asset_server
            .load("third-party/audio/other/explosionCrunch_000.ogg");

        let despawn_effect = ParticleEffectBundle::new(builder.effects.add(death_effect())).into();

        let (outline_mesh, outline_material) = Self::outline(builder, color);
        CharacterAssets {
            scene: SceneRoot(model),
            outline_mesh,
            outline_material,
            despawn_sound,
            despawn_effect,
        }
    }

    pub fn player(builder: &mut Builder) -> Self {
        Self::character(builder, GREEN.into(), "models/temp/robot1.glb#Scene0")
    }

    pub fn ally(builder: &mut Builder) -> Self {
        Self::character(builder, LIGHT_CYAN.into(), "models/temp/robot1.glb#Scene0")
    }

    pub fn enemy(builder: &mut Builder) -> Self {
        Self::character(builder, RED.into(), "models/temp/snowman.glb#Scene0")
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

    let spawner = Spawner::once(500.0.into(), true);
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
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
}
