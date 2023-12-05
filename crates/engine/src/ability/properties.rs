use std::f32::consts::PI;

use bevy_ecs::system::Resource;

use crate::{death_callback::ExplosionKind, movement::MaxSpeed, time::Tick};

use super::{grenade::GrenadeKind, Ability};

#[derive(Debug, Resource)]
pub struct AbilityProps {
    pub hyper_sprint: HyperSprintProps,
    pub gun: GunProps,
    pub shotgun: ShotgunProps,
    pub frag_grenade: GrenadeProps,
    pub heal_grenade: GrenadeProps,
    pub seeker_rocket: SeekerRocketProps,
    pub neutrino_ball: NeutrinoBallProps,
}

impl Default for AbilityProps {
    fn default() -> Self {
        Self {
            frag_grenade: GrenadeProps {
                cost: 30.0,
                cooldown: Tick(30),
                delay: Tick(120),
                radius: 0.07,
                kind: GrenadeKind::Frag,
                health: 3.0,
                explosion: ExplosionProps {
                    min_radius: 0.3,
                    max_radius: 1.8,
                    duration: Tick(15),
                    damage: 0.6,
                    force: 8.0,
                    kind: ExplosionKind::FragGrenade,
                },
            },
            heal_grenade: GrenadeProps {
                cost: 50.0,
                cooldown: Tick(30),
                delay: Tick(120),
                radius: 0.05,
                kind: GrenadeKind::Heal,
                health: 3.0,
                explosion: ExplosionProps {
                    min_radius: 0.2,
                    max_radius: 1.2,
                    duration: Tick(15),
                    damage: -1.5,
                    force: 0.0,
                    kind: ExplosionKind::HealGrenade,
                },
            },
            hyper_sprint: Default::default(),
            gun: Default::default(),
            shotgun: Default::default(),
            seeker_rocket: Default::default(),
            neutrino_ball: Default::default(),
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
            Ability::NeutrinoBall => self.neutrino_ball.cooldown,
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
            Ability::NeutrinoBall => self.neutrino_ball.cost,
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
            factor: 1.8,
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
            cooldown: Tick(5),
            duration: Tick(600),
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
    pub cooldown: Tick,
    pub delay: Tick,
    pub radius: f32,
    pub kind: GrenadeKind,
    pub health: f32,
    pub explosion: ExplosionProps,
}

#[derive(Debug)]
pub struct SeekerRocketProps {
    pub cost: f32,
    pub cooldown: Tick,
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
            cooldown: Tick(30),
            turning_radius: PI * 0.03,
            capsule_radius: 0.05,
            capsule_length: 0.14,
            health: 3.0,
            max_speed: MaxSpeed {
                force: 3.0,
                speed: 8.0,
            },
            energy: 10.0,
            energy_cost: 0.2,
            explosion: ExplosionProps {
                min_radius: 0.2,
                max_radius: 1.2,
                duration: Tick(15),
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
    pub duration: Tick,
    pub damage: f32,
    pub force: f32,
    pub kind: ExplosionKind,
}

#[derive(Debug, Copy, Clone)]
pub struct NeutrinoBallProps {
    pub cost: f32,
    pub cooldown: Tick,
    pub radius: f32,
    pub effect_radius: f32,
    pub duration: Tick,
    pub activation_delay: Tick,
    pub speed: f32,
    pub surface_a: f32,
}

impl Default for NeutrinoBallProps {
    fn default() -> Self {
        Self {
            cost: 50.0,
            cooldown: Tick(30),
            radius: 0.3,
            effect_radius: 2.0,
            duration: Tick(240),
            activation_delay: Tick(45),
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
