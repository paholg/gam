use std::f32::consts::PI;

use bevy_ecs::system::Resource;

use crate::{death_callback::ExplosionKind, movement::MaxSpeed, time::Dur};

use super::{grenade::GrenadeKind, Ability};

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
                    force: 8.0,
                    kind: ExplosionKind::FragGrenade,
                },
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
pub struct GunProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub duration: Dur,
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
            cooldown: Dur::new(5),
            duration: Dur::new(600),
            speed: 12.0,
            radius: 0.03,
            damage: 2.0,
            bullet_health: 1.0,
            density: 100.0,
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
    pub density: f32,
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
            density: 100.0,
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
}

#[derive(Debug)]
pub struct SeekerRocketProps {
    pub cost: f32,
    pub cooldown: Dur,
    /// Max turn per tick, in radians.
    pub turning_radius: f32,
    pub capsule_radius: f32,
    pub capsule_length: f32,
    pub health: f32,
    pub max_speed: MaxSpeed,
    /// How much energy the rocket has.
    pub energy: f32,
    /// How much energy the rocket spends every frame to move.
    pub energy_cost: f32,
    pub explosion: ExplosionProps,
}

impl Default for SeekerRocketProps {
    fn default() -> Self {
        Self {
            cost: 30.0,
            cooldown: Dur::new(30),
            turning_radius: PI * 0.03,
            capsule_radius: 0.05,
            capsule_length: 0.14,
            health: 3.0,
            max_speed: MaxSpeed {
                accel: 1800.0,
                speed: 8.0,
            },
            energy: 10.0,
            energy_cost: 0.2,
            explosion: ExplosionProps {
                min_radius: 0.2,
                max_radius: 1.2,
                duration: Dur::new(15),
                damage: 0.6,
                force: 6.0,
                kind: ExplosionKind::SeekerRocket,
            },
        }
    }
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
pub struct NeutrinoBallProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub radius: f32,
    pub effect_radius: f32,
    pub duration: Dur,
    pub activation_delay: Dur,
    pub speed: f32,
    pub surface_a: f32,
}

impl Default for NeutrinoBallProps {
    fn default() -> Self {
        Self {
            cost: 50.0,
            cooldown: Dur::new(30),
            radius: 0.3,
            effect_radius: 2.0,
            duration: Dur::new(240),
            activation_delay: Dur::new(30),
            speed: 3.0,
            surface_a: 300.0,
        }
    }
}

impl NeutrinoBallProps {
    /// The acceleration due to this ball will be the result value, divided by
    /// distance squared.
    pub fn accel_numerator(&self) -> f32 {
        self.surface_a * self.radius * self.radius
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TransportProps {
    pub cost: f32,
    pub cooldown: Dur,
    pub radius: f32,
    pub height: f32,
    pub accel: f32,
    pub speed: f32,
    pub delay: Dur,
}

impl Default for TransportProps {
    fn default() -> Self {
        Self {
            cost: 40.0,
            cooldown: Dur::new(90),
            radius: 0.5,
            height: 2.0,
            accel: 100.0,
            speed: 3.0,
            delay: Dur::new(90),
        }
    }
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
            cost: 1.0,
            cooldown: Dur::new(1),
            duration: Dur::new(1),
            amount: 1.0,
        }
    }
}
