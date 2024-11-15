use std::ops::Index;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Commands;
use bevy_math::Quat;
use bevy_rapier3d::prelude::Velocity;
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumIter;

use self::bullet::Bullet;
use self::bullet::BulletSpawner;
use self::grenade::grenade;
use self::neutrino_ball::neutrino_ball;
use self::properties::AbilityProps;
use self::properties::GunProps;
use self::properties::ShotgunProps;
use self::seeker_rocket::seeker_rocket;
use self::speed_up::speed_up;
use self::transport::transport;
use crate::status_effect::TimeDilation;
use crate::AbilityOffset;
use crate::Cooldowns;
use crate::Energy;
use crate::FootOffset;
use crate::Health;
use crate::Target;
use crate::FORWARD;
use crate::PLAYER_R;

pub mod bullet;
pub mod grenade;
pub mod neutrino_ball;
pub mod properties;
pub mod seeker_rocket;
pub mod speed_up;
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
    Gun,
    Shotgun,
    FragGrenade,
    HealGrenade,
    SeekerRocket,
    NeutrinoBall,
    Transport,
    SpeedUp,
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
        props: &AbilityProps,
        entity: Entity,
        energy: &mut Energy,
        cooldowns: &mut Cooldowns,
        transform: &Transform,
        velocity: &Velocity,
        target: &Target,
        ability_offset: &AbilityOffset,
        _foot_offset: &FootOffset,
        time_dilation: &mut TimeDilation,
    ) -> bool {
        if cooldowns.is_available(self, time_dilation) && energy.try_use(props.cost(self)) {
            cooldowns.set(*self, props.cooldown(self));
        } else {
            return false;
        }

        match self {
            Ability::None => (),
            Ability::Gun => gun(
                commands,
                &props.gun,
                transform,
                velocity,
                entity,
                ability_offset,
            ),
            Ability::Shotgun => shotgun(
                commands,
                &props.shotgun,
                transform,
                velocity,
                entity,
                ability_offset,
            ),
            Ability::FragGrenade => grenade(
                commands,
                &props.frag_grenade,
                transform,
                entity,
                target,
                ability_offset,
            ),
            Ability::HealGrenade => grenade(
                commands,
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
            ),
            Ability::Transport => transport(commands, entity, &props.transport, transform, target),
            Ability::SpeedUp => speed_up(&props.speed_up, time_dilation),
        }
        true
    }
}

fn gun(
    commands: &mut Commands,
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
        mass: props.mass,
        bullet: Bullet {
            shooter,
            expires_in: props.duration,
            damage: props.damage,
        },
        health: Health::new(props.bullet_health),
    }
    .spawn(commands);
}

fn shotgun(
    commands: &mut Commands,
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
            mass: props.mass,
            bullet: Bullet {
                shooter,
                expires_in: props.duration,
                damage: props.damage,
            },
            health: Health::new(props.bullet_health),
        }
        .spawn(commands);
    }
}
