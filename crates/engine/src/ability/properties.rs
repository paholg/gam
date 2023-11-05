use std::{f32::consts::PI, time::Duration};

use bevy_ecs::system::Resource;

use crate::time::Tick;

use super::Ability;

#[derive(Debug, Resource)]
pub struct AbilityProps {
    pub hyper_sprint: HyperSprintProps,
    pub gun: GunProps,
    pub shotgun: ShotgunProps,
    pub frag_grenade: GrenadeProps,
    pub heal_grenade: GrenadeProps,
}

impl Default for AbilityProps {
    fn default() -> Self {
        Self {
            frag_grenade: GrenadeProps {
                cost: 20.0,
                cooldown: Tick(30),
                delay: Tick(120),
                damage: 8.0,
                explosion_radius: 7.0,
                radius: 0.30,
                kind: GrenadeKind::Frag,
            },
            heal_grenade: GrenadeProps {
                cost: 50.0,
                cooldown: Tick(30),
                delay: Tick(120),
                damage: -20.0,
                explosion_radius: 4.0,
                radius: 0.20,
                kind: GrenadeKind::Heal,
            },
            hyper_sprint: Default::default(),
            gun: Default::default(),
            shotgun: Default::default(),
        }
    }
}

impl AbilityProps {
    pub fn cooldown(&self, ability: &Ability) -> Tick {
        match ability {
            Ability::None => Tick::default(),
            Ability::HyperSprint => Tick::default(),
            Ability::Gun => self.gun.cooldown,
            Ability::Shotgun => self.shotgun.cooldown,
            Ability::FragGrenade => self.frag_grenade.cooldown,
            Ability::HealGrenade => self.heal_grenade.cooldown,
        }
    }

    pub fn cost(&self, ability: &Ability) -> f32 {
        match ability {
            Ability::None => 0.0,
            Ability::HyperSprint => self.hyper_sprint.cost,
            Ability::Gun => self.gun.cost,
            Ability::Shotgun => self.shotgun.cost,
            Ability::FragGrenade => self.frag_grenade.cost,
            Ability::HealGrenade => self.heal_grenade.cost,
        }
    }
}

#[derive(Debug)]
pub struct HyperSprintProps {
    pub cost: f32,
    /// Speed multiplication factor.
    pub factor: f32,
}

impl Default for HyperSprintProps {
    fn default() -> Self {
        Self {
            cost: 2.0,
            factor: 7.0,
        }
    }
}

#[derive(Debug)]
pub struct GunProps {
    pub cost: f32,
    pub cooldown: Tick,
    pub duration: Tick,
    pub speed: f32,
    pub radius: f32,
    pub damage: f32,
    pub density: f32,
}

impl Default for GunProps {
    fn default() -> Self {
        Self {
            cost: 5.0,
            cooldown: Tick::new(Duration::from_millis(150)),
            duration: Tick::new(Duration::from_secs(10)),
            speed: 50.0,
            radius: 0.15,
            damage: 1.0,
            density: 100.0,
        }
    }
}

#[derive(Debug)]
pub struct ShotgunProps {
    pub cost: f32,
    pub cooldown: Tick,
    pub duration: Tick,
    pub speed: f32,
    pub radius: f32,
    pub damage: f32,
    pub n_pellets: usize,
    /// Spread angle in radians.
    pub spread: f32,
    pub density: f32,
}

impl Default for ShotgunProps {
    fn default() -> Self {
        Self {
            cost: 25.0,
            cooldown: Tick::new(Duration::from_millis(150)),
            duration: Tick::new(Duration::from_secs(10)),
            speed: 30.0,
            radius: 0.15,
            damage: 1.0,
            n_pellets: 8,
            spread: PI * 0.125,
            density: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GrenadeKind {
    Frag,
    Heal,
}

#[derive(Debug)]
pub struct GrenadeProps {
    pub cost: f32,
    pub cooldown: Tick,
    pub delay: Tick,
    pub damage: f32,
    pub explosion_radius: f32,
    pub radius: f32,
    pub kind: GrenadeKind,
}
