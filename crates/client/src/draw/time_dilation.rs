use bevy::{
    core::FrameCount,
    prelude::{Commands, Query, Res, ResMut, Transform, Without},
};
use bevy_hanabi::EffectSpawner;
use engine::{status_effect::TimeDilation, FootOffset};

use crate::asset_handler::AssetHandler;

pub fn draw_time_dilation_system(
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
