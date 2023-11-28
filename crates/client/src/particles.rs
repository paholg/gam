use bevy::{
    core::FrameCount,
    prelude::{Commands, Entity, Query, Transform},
};
use bevy_hanabi::{EffectSpawner, ParticleEffectBundle};

/// A wrapper around `ParticleEffectBundle` that allows spawning multiple copies
/// of the same effect in the same frame.
///
/// Some caveats:
/// 1. It is expected that the effect's spawner has `starts_immediately: true`.
///    This is left to the caller to verify.
pub struct ParticleEffectPool {
    bundle: ParticleEffectBundle,
    effects: Vec<Entity>,
    index: usize,
    last_run: u32,
}

impl Clone for ParticleEffectPool {
    fn clone(&self) -> Self {
        Self::new(self.bundle.clone())
    }
}

impl From<ParticleEffectBundle> for ParticleEffectPool {
    fn from(value: ParticleEffectBundle) -> Self {
        Self::new(value)
    }
}

impl ParticleEffectPool {
    pub fn new(bundle: ParticleEffectBundle) -> Self {
        Self {
            bundle,
            effects: vec![],
            index: 0,
            last_run: 0,
        }
    }

    /// Each new frame, we can re-use `ParticleEffect`s.
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

        tracing::info!(i = self.index, l = self.effects.len());

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
            let mut bundle = self.bundle.clone();
            bundle.transform = transform;
            let entity = commands.spawn(bundle).id();
            self.effects.push(entity);
            self.index += 1;
        }
    }
}
