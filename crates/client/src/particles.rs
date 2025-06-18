use bevy::asset::Handle;
use bevy::diagnostic::FrameCount;
use bevy::prelude::Commands;
use bevy::prelude::Entity;
use bevy::prelude::Query;
use bevy::prelude::Transform;
use bevy_hanabi::EffectAsset;
use bevy_hanabi::EffectSpawner;
use bevy_hanabi::ParticleEffect;

// A wrapper around `Handle<EffectAsset>undle` that allows spawning multiple copies
/// of the same effect in the same frame.
///
/// Some caveats:
/// 1. It is expected that the effect's spawner has `starts_immediately: true`.
///    This is left to the caller to verify.
pub struct ParticleEffectPool {
    asset: Handle<EffectAsset>,
    effects: Vec<Entity>,
    index: usize,
    last_run: u32,
}

impl From<Handle<EffectAsset>> for ParticleEffectPool {
    fn from(value: Handle<EffectAsset>) -> Self {
        Self::new(value)
    }
}

impl ParticleEffectPool {
    pub fn new(effect: Handle<EffectAsset>) -> Self {
        Self {
            asset: effect,
            effects: vec![],
            index: 0,
            last_run: 0,
        }
    }

    /// Each new frame, we can re-use `Handle<EffectAsset>s.
    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn trigger(
        &mut self,
        commands: &mut Commands,
        transform: Transform,
        effects: &mut Query<(&mut Transform, &mut EffectSpawner)>,
        frame: &FrameCount,
    ) {
        if self.last_run != frame.0 {
            self.reset();
        }
        self.last_run = frame.0;

        if self.index < self.effects.len() {
            let entity = self.effects[self.index];
            self.index += 1;
            if let Ok((mut effect_transform, mut effect_spawner)) = effects.get_mut(entity) {
                *effect_transform = transform;
                effect_spawner.reset();
            } else {
                tracing::warn!("Missing effect");
            }
        } else {
            let effect = ParticleEffect {
                handle: self.asset.clone(),
                prng_seed: None,
            };
            let entity = commands.spawn((effect, transform)).id();
            self.effects.push(entity);
            self.index += 1;
        }
    }
}
