use bevy::asset::Assets;
use bevy::diagnostic::FrameCount;
use bevy::ecs::resource::Resource;
use bevy::math::Vec3;
use bevy::math::Vec4;
use bevy::prelude::Commands;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Transform;
use bevy::prelude::Without;
use bevy_hanabi::Attribute;
use bevy_hanabi::ColorOverLifetimeModifier;
use bevy_hanabi::EffectAsset;
use bevy_hanabi::EffectSpawner;
use bevy_hanabi::ExprWriter;
use bevy_hanabi::Gradient;
use bevy_hanabi::SetAttributeModifier;
use bevy_hanabi::SetPositionCircleModifier;
use bevy_hanabi::SetVelocityCircleModifier;
use bevy_hanabi::ShapeDimension;
use bevy_hanabi::SizeOverLifetimeModifier;
use bevy_hanabi::SpawnerSettings;
use engine::status_effect::TimeDilation;
use engine::FootOffset;
use engine::PLAYER_R;

use crate::particles::ParticleEffectPool;

#[derive(Resource)]
pub struct TimeDilationAssets {
    fast_effect: ParticleEffectPool,
}

impl TimeDilationAssets {
    fn new(effects: &mut Assets<EffectAsset>) -> Self {
        let effect = effects.add(fast_effect());
        let fast_effect = effect.into();

        TimeDilationAssets { fast_effect }
    }
}

fn fast_effect() -> EffectAsset {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::splat(1.0));
    color_gradient.add_key(0.5, Vec4::splat(1.0));
    color_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let spawner = SpawnerSettings::once(32.0.into());
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
            ..Default::default()
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant([0.05; 3].into()),
            screen_space_size: false,
        })
}

pub fn setup_time_dilation(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    commands.insert_resource(TimeDilationAssets::new(&mut effects));
}

pub fn draw_time_dilation_system(
    mut commands: Commands,
    mut assets: ResMut<TimeDilationAssets>,
    mut effects: Query<(&mut Transform, &mut EffectSpawner)>,
    query: Query<(&Transform, &FootOffset, &TimeDilation), Without<EffectSpawner>>,
    frame: Res<FrameCount>,
) {
    let effect = &mut assets.fast_effect;

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
