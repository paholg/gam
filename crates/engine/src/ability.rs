use std::ops::Index;

use bevy_ecs::{component::Component, entity::Entity, system::Commands};
use bevy_math::Quat;
use bevy_rapier3d::prelude::Velocity;
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tracing::warn;

use crate::{
    status_effect::{StatusEffect, StatusEffects},
    time::TickCounter,
    AbilityOffset, Cooldowns, Energy, Health, Target, FORWARD, PLAYER_R,
};

use self::{
    bullet::{Bullet, BulletSpawner},
    grenade::grenade,
    neutrino_ball::neutrino_ball,
    properties::{AbilityProps, GunProps, ShotgunProps},
    seeker_rocket::seeker_rocket,
    transport::transport,
};

pub mod bullet;
pub mod grenade;
pub mod neutrino_ball;
pub mod properties;
pub mod seeker_rocket;
pub mod transport;

#[derive(
    Debug,
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    EnumIter,
    Display,
    Reflect,
    Hash,
)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
    Gun,
    Shotgun,
    FragGrenade,
    HealGrenade,
    SeekerRocket,
    NeutrinoBall,
    Transport,
}

#[derive(Debug, Component, Clone, Serialize, Deserialize)]
pub struct Abilities {
    inner: Vec<Ability>,
}

impl Abilities {
    pub fn new(abilities: Vec<Ability>) -> Self {
        Self { inner: abilities }
    }

    pub fn iter(&self) -> impl Iterator<Item = Ability> + '_ {
        self.inner.iter().copied()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Ability> {
        self.inner.iter_mut()
    }
}

impl Index<usize> for Abilities {
    type Output = Ability;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl Ability {
    #[allow(clippy::too_many_arguments)]
    pub fn fire(
        &self,
        commands: &mut Commands,
        tick_counter: &TickCounter,
        props: &AbilityProps,
        entity: Entity,
        energy: &mut Energy,
        cooldowns: &mut Cooldowns,
        transform: &Transform,
        velocity: &Velocity,
        status_effects: &mut StatusEffects,
        target: &Target,
        ability_offset: &AbilityOffset,
    ) -> bool {
        let cooldown = match cooldowns.get_mut(self) {
            Some(cd) => cd,
            None => {
                warn!("Tried to use an ability that we don't have a cooldown for");
                return false;
            }
        };

        if cooldown.before_now(tick_counter) && energy.try_use(props.cost(self)) {
            *cooldown = tick_counter.at(props.cooldown(self));
        } else {
            return false;
        }

        match self {
            Ability::None => (),
            Ability::HyperSprint => hyper_sprint(commands, entity, status_effects),
            Ability::Gun => gun(
                commands,
                tick_counter,
                &props.gun,
                transform,
                velocity,
                entity,
                ability_offset,
            ),
            Ability::Shotgun => shotgun(
                commands,
                tick_counter,
                &props.shotgun,
                transform,
                velocity,
                entity,
                ability_offset,
            ),
            Ability::FragGrenade => grenade(
                commands,
                tick_counter,
                &props.frag_grenade,
                transform,
                entity,
                target,
                ability_offset,
            ),
            Ability::HealGrenade => grenade(
                commands,
                tick_counter,
                &props.heal_grenade,
                transform,
                entity,
                target,
                ability_offset,
            ),
            Ability::SeekerRocket => seeker_rocket(
                commands,
                &props.seeker_rocket,
                transform,
                velocity,
                entity,
                ability_offset,
            ),
            Ability::NeutrinoBall => neutrino_ball(
                commands,
                &props.neutrino_ball,
                transform,
                velocity,
                ability_offset,
                tick_counter,
            ),
            Ability::Transport => transport(
                commands,
                entity,
                &props.transport,
                transform,
                target,
                tick_counter,
            ),
        }
        true
    }

    pub fn unfire(
        &self,
        commands: &mut Commands,
        entity: Entity,
        status_effects: &mut StatusEffects,
    ) {
        match self {
            Ability::HyperSprint => {
                hyper_sprint_disable(commands, entity, status_effects);
            }
            Ability::None
            | Ability::Gun
            | Ability::Shotgun
            | Ability::FragGrenade
            | Ability::HealGrenade
            | Ability::SeekerRocket
            | Ability::NeutrinoBall
            | Ability::Transport => (),
        }
    }
}

#[derive(Component, Hash)]
pub struct HyperSprinting;

fn hyper_sprint(commands: &mut Commands, entity: Entity, status_effects: &mut StatusEffects) {
    commands.entity(entity).insert(HyperSprinting);
    status_effects.effects.insert(StatusEffect::HyperSprinting);
}

fn hyper_sprint_disable(
    commands: &mut Commands,
    entity: Entity,
    status_effects: &mut StatusEffects,
) {
    status_effects.effects.remove(&StatusEffect::HyperSprinting);
    commands.entity(entity).remove::<HyperSprinting>();
}

fn gun(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &GunProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
    ability_offset: &AbilityOffset,
) {
    let dir = transform.rotation * FORWARD;
    let position =
        transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ability_offset.to_vec();
    let velocity = dir * props.speed + velocity.linvel;
    BulletSpawner {
        position,
        velocity,
        radius: props.radius,
        density: props.density,
        bullet: Bullet {
            shooter,
            duration: tick_counter.at(props.duration),
            damage: props.damage,
        },
        health: Health::new(props.bullet_health),
    }
    .spawn(commands);
}

fn shotgun(
    commands: &mut Commands,
    tick_counter: &TickCounter,
    props: &ShotgunProps,
    transform: &Transform,
    velocity: &Velocity,
    shooter: Entity,
    ability_offset: &AbilityOffset,
) {
    for i in 0..props.n_pellets {
        let idx = i as f32;
        let n_pellets = props.n_pellets as f32;
        let relative_angle = (n_pellets * 0.5 - idx) / n_pellets * props.spread;
        let relative_angle = Quat::from_rotation_z(relative_angle);
        let dir = (transform.rotation * relative_angle) * FORWARD;
        let position =
            transform.translation + dir * (PLAYER_R + props.radius * 2.0) + ability_offset.to_vec();
        let velocity = dir * props.speed + velocity.linvel;
        BulletSpawner {
            position,
            velocity,
            radius: props.radius,
            density: props.density,
            bullet: Bullet {
                shooter,
                duration: tick_counter.at(props.duration),
                damage: props.damage,
            },
            health: Health::new(props.bullet_health),
        }
        .spawn(commands);
    }
}
