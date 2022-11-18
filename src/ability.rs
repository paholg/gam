use std::time::Duration;

use bevy::{
    prelude::{
        Assets, Commands, Component, Entity, Mesh, Query, Res, ResMut, StandardMaterial, Transform,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_rapier3d::prelude::Velocity;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{MaxSpeed, PlayerCooldowns};

/// Construct a cooldown timer
pub fn cooldown(cooldown: Duration) -> Timer {
    let mut timer = Timer::new(cooldown, TimerMode::Once);
    timer.tick(cooldown);
    timer
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Shoot,
}

impl Ability {
    pub fn fire(
        &self,
        commands: &mut Commands,
        entity: Entity,
        cooldowns: &mut PlayerCooldowns,
        max_speed: &mut MaxSpeed,
    ) {
        match self {
            Ability::None => (),
            Ability::HyperSprint => hyper_sprint(commands, entity, cooldowns, max_speed),
            Ability::Shoot => todo!(),
        }
    }
}

#[derive(Component)]
pub struct HyperSprinting {
    duration: Timer,
}

const HYPER_SPRINT_FACTOR: f32 = 5.0;
pub const HYPER_SPRINT_COOLDOWN: Duration = Duration::new(5, 0);

fn hyper_sprint(
    commands: &mut Commands,
    entity: Entity,
    cooldowns: &mut PlayerCooldowns,
    max_speed: &mut MaxSpeed,
) {
    if cooldowns.hyper_sprint.finished() {
        cooldowns.hyper_sprint.reset();
        max_speed.0 *= HYPER_SPRINT_FACTOR;
        commands.entity(entity).insert(HyperSprinting {
            duration: Timer::from_seconds(0.15, TimerMode::Once),
        });
    } else {
        info!(
            hyper_sprint = cooldowns.hyper_sprint.remaining_secs(),
            "Remaining"
        );
    }
}

pub fn hyper_sprint_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HyperSprinting, &mut MaxSpeed)>,
    time: Res<Time>,
) {
    for (entity, mut hyper_sprinting, mut max_speed) in query.iter_mut() {
        if hyper_sprinting.duration.tick(time.delta()).just_finished() {
            max_speed.0 /= HYPER_SPRINT_FACTOR;
            commands.entity(entity).remove::<HyperSprinting>();
        }
    }
}

pub const SHOOT_COOLDOWN: Duration = Duration::from_millis(500);

fn shoot(
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    _transform: &Transform,
    _velocity: &Velocity,
) {
}
