use bevy::prelude::Vec3;
use bevy::prelude::Vec4;
use bevy_hanabi::Attribute;
use bevy_hanabi::ColorOverLifetimeModifier;
use bevy_hanabi::EffectAsset;
use bevy_hanabi::ExprWriter;
use bevy_hanabi::Gradient;
use bevy_hanabi::ParticleEffectBundle;
use bevy_hanabi::SetAttributeModifier;
use bevy_hanabi::SetPositionCircleModifier;
use bevy_hanabi::SetVelocityCircleModifier;
use bevy_hanabi::ShapeDimension;
use bevy_hanabi::SizeOverLifetimeModifier;
use bevy_hanabi::Spawner;
use engine::PLAYER_R;

use super::Builder;
use crate::particles::ParticleEffectPool;

pub struct TimeDilationAssets {
    pub fast_effect: ParticleEffectPool,
}

impl TimeDilationAssets {
    pub fn new(builder: &mut Builder) -> Self {
        let effect = builder.effects.add(fast_effect());
        let fast_effect = ParticleEffectBundle::new(effect).into();

        TimeDilationAssets { fast_effect }
    }
}

fn fast_effect() -> EffectAsset {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::splat(1.0));
    color_gradient.add_key(0.5, Vec4::splat(1.0));
    color_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let spawner = Spawner::once(32.0.into(), true);
    let writer = ExprWriter::new();

    let pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::Y * 0.1).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(PLAYER_R).expr(),
        dimension: ShapeDimension::Surface,
    };

    let vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed: writer.lit(0.5).uniform(writer.lit(0.8)).expr(),
    };

    let lifetime = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(0.5).expr(),
    };

    EffectAsset::new(32768, spawner, writer.finish())
        .with_name("hyper_sprint_effect")
        .init(pos)
        .init(vel)
        .init(lifetime)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant([0.05; 2].into()),
            screen_space_size: false,
        })
}
