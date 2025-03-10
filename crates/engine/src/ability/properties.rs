use std::f32::consts::PI;

use bevy_ecs::system::Resource;

use super::grenade::GrenadeKind;
use super::Ability;
use crate::death_callback::ExplosionKind;
use crate::movement::MaxSpeed;
use crate::time::Dur;

#[derive(Debug, Resource)]
pub struct AbilityProps {
    pub gun: GunProps,
    pub shotgun: ShotgunProps,
    pub frag_grenade: GrenadeProps,
    pub heal_grenade: GrenadeProps,
    pub seeker_rocket: SeekerRocketProps,
    pub neutrino_ball: NeutrinoBallProps,
    pub transport: TransportProps,
    pub speed_up: SpeedUpProps,
}

impl Default for AbilityProps {
    fn default() -> Self {
        Self {
            frag_grenade: GrenadeProps {
                cost: 30.0,
                cooldown: Dur::new(30),
                delay: Dur::new(120),
                radius: 0.07,
                kind: GrenadeKind::Frag,
                health: 3.0,
                explosion: ExplosionProps {
                    min_radius: 0.3,
                    max_radius: 1.8,
                    duration: Dur::new(15),
                    damage: 0.6,
                    force: 400.0,
                    kind: ExplosionKind::FragGrenade,
                },
                mass: 1.5,
            },
            heal_grenade: GrenadeProps {
                cost: 50.0,
                cooldown: Dur::new(30),
                delay: Dur::new(120),
                radius: 0.05,
                kind: GrenadeKind::Heal,
                health: 3.0,
                explosion: ExplosionProps {
                    min_radius: 0.2,
                    max_radius: 1.2,
                    duration: Dur::new(15),
                    damage: -1.5,
                    force: 0.0,
                    kind: ExplosionKind::HealGrenade,
                },
                mass: 1.0,
            },
            gun: Default::default(),
            shotgun: Default::default(),
            seeker_rocket: Default::default(),
            neutrino_ball: Default::default(),
            transport: Default::default(),
            speed_up: SpeedUpProps::default(),
        }
    }
}

impl AbilityProps {
    pub fn cooldown(&self, ability: &Ability) -> Dur {
        match ability {
            Ability::None => Dur::default(),
            Ability::Gun => self.gun.cooldown,
            Ability::Shotgun => self.shotgun.cooldown,
            Ability::FragGrenade => self.frag_grenade.cooldown,
            Ability::HealGrenade => self.heal_grenade.cooldown,
            Ability::SeekerRocket => self.seeker_rocket.cooldown,
            Ability::NeutrinoBall => self.neutrino_ball.cooldown,
            Ability::Transport => self.transport.cooldown,
            Ability::SpeedUp => self.speed_up.cooldown,
        }
    }

    pub fn cost(&self, ability: &Ability) -> f32 {
        match ability {
            Ability::None => 0.0,
            Ability::Gun => self.gun.cost,
            Ability::Shotgun => self.shotgun.cost,
            Ability::FragGrenade => self.frag_grenade.cost,
            Ability::HealGrenade => self.heal_grenade.cost,
            Ability::SeekerRocket => self.seeker_rocket.cost,
            Ability::NeutrinoBall => self.neutrino_ball.cost,
            Ability::Transport => self.transport.cost,
            Ability::SpeedUp => self.speed_up.cost,
        }
    }
}
#[derive(Debug)]
pub struct ShotgunProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub duration: Dur,
    pub speed: f32,
    pub radius: f32,
    pub damage: f32,
    pub n_pellets: usize,
    /// Spread angle in radians.
    pub spread: f32,
    pub mass: f32,
    pub bullet_health: f32,
}

impl Default for ShotgunProps {
    fn default() -> Self {
        Self {
            cost: 25.0,
            cooldown: Dur::new(10),
            duration: Dur::new(600),
            speed: 12.0,
            radius: 0.03,
            damage: 1.0,
            bullet_health: 1.0,
            n_pellets: 8,
            spread: PI * 0.125,
            mass: 0.25,
        }
    }
}

#[derive(Debug)]
pub struct GrenadeProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub delay: Dur,
    pub radius: f32,
    pub kind: GrenadeKind,
    pub health: f32,
    pub explosion: ExplosionProps,
    pub mass: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct ExplosionProps {
    pub min_radius: f32,
    pub max_radius: f32,
    pub duration: Dur,
    pub damage: f32,
    pub force: f32,
    pub kind: ExplosionKind,
}

#[derive(Debug, Copy, Clone)]
pub struct SpeedUpProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub duration: Dur,
    pub amount: f32,
}

impl Default for SpeedUpProps {
    fn default() -> Self {
        Self {
            cost: 2.0,
            cooldown: Dur::new(1),
            duration: Dur::new(1),
            amount: 1.0,
        }
    }
}
