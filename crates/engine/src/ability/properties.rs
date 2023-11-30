use std::f32::consts::PI;

use bevy_ecs::system::Resource;

use crate::{movement::MaxSpeed, time::Tick};

use super::{grenade::GrenadeKind, Ability};

#[derive(Debug, Resource)]
pub struct AbilityProps {
    pub hyper_sprint: HyperSprintProps,
    pub gun: GunProps,
    pub shotgun: ShotgunProps,
    pub frag_grenade: GrenadeProps,
    pub heal_grenade: GrenadeProps,
    pub seeker_rocket: SeekerRocketProps,
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
                health: 3.0,
            },
            heal_grenade: GrenadeProps {
                cost: 50.0,
                cooldown: Tick(30),
                delay: Tick(120),
                damage: -20.0,
                explosion_radius: 4.0,
                radius: 0.20,
                kind: GrenadeKind::Heal,
                health: 3.0,
            },
            hyper_sprint: Default::default(),
            gun: Default::default(),
            shotgun: Default::default(),
            seeker_rocket: Default::default(),
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
            Ability::SeekerRocket => self.seeker_rocket.cooldown,
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
            Ability::SeekerRocket => self.seeker_rocket.cost,
        }
    }
}

#[derive(Debug)]
pub struct HyperSprintProps {
    pub cost: f32,
    /// Speed multiplication factor.
    pub factor: f32,
    pub cooldown: Tick,
}

impl Default for HyperSprintProps {
    fn default() -> Self {
        Self {
            cost: 2.0,
            factor: 5.0,
            cooldown: Tick(0),
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
    pub bullet_health: f32,
}

impl Default for GunProps {
    fn default() -> Self {
        Self {
            cost: 5.0,
            cooldown: Tick(10),
            duration: Tick(600),
            speed: 50.0,
            radius: 0.15,
            damage: 1.0,
            bullet_health: 1.0,
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
    pub bullet_health: f32,
}

impl Default for ShotgunProps {
    fn default() -> Self {
        Self {
            cost: 25.0,
            cooldown: Tick(10),
            duration: Tick(600),
            speed: 30.0,
            radius: 0.15,
            damage: 1.0,
            bullet_health: 1.0,
            n_pellets: 8,
            spread: PI * 0.125,
            density: 100.0,
        }
    }
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
    pub health: f32,
}

#[derive(Debug)]
pub struct SeekerRocketProps {
    pub cost: f32,
    pub cooldown: Tick,
    pub duration: Tick,
    pub damage: f32,
    pub explosion_radius: f32,
    // Max turn per tick, in radians.
    pub turning_radius: f32,
    // Note: Shape is a capsule.
    pub radius: f32,
    pub length: f32,
    pub health: f32,
    pub max_speed: MaxSpeed,
}

impl Default for SeekerRocketProps {
    fn default() -> Self {
        Self {
            cost: 20.0,
            cooldown: Tick(30),
            duration: Tick(300),
            damage: 8.0,
            explosion_radius: 3.0,
            turning_radius: PI * 0.02,
            radius: 0.3,
            length: 0.6,
            health: 3.0,
            max_speed: MaxSpeed {
                force: 20.0,
                speed: 40.0,
            },
        }
    }
}
