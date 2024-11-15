use bevy::core::FrameCount;
use bevy::prelude::Commands;
use bevy::prelude::Query;
use bevy::prelude::Res;
use bevy::prelude::ResMut;
use bevy::prelude::Transform;
use bevy::prelude::Without;
use bevy_hanabi::EffectInitializers;
use engine::status_effect::TimeDilation;
use engine::FootOffset;

use crate::asset_handler::AssetHandler;

pub fn draw_time_dilation_system(
    mut commands: Commands,
    mut assets: ResMut<AssetHandler>,
    mut effects: Query<(&mut Transform, &mut EffectInitializers)>,
    query: Query<(&Transform, &FootOffset, &TimeDilation), Without<EffectInitializers>>,
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
