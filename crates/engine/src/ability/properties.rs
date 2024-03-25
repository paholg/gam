use std::f32::consts::PI;

use crate::{death_callback::ExplosionKind, time::Dur};

// #[derive(Debug, Resource)]
// pub struct AbilityProps {
//     pub gun: GunProps,
//     pub shotgun: ShotgunProps,
//     pub frag_grenade: GrenadeProps,
//     pub heal_grenade: GrenadeProps,
//     pub seeker_rocket: SeekerRocketProps,
//     pub neutrino_ball: NeutrinoBallProps,
//     pub transport: TransportProps,
//     pub speed_up: SpeedUpProps,
// }

// impl Default for AbilityProps {
//     fn default() -> Self {
//         Self {
//     }
// }

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
            cost: 2.0,
            cooldown: Dur::new(1),
            duration: Dur::new(1),
            amount: 1.0,
        }
    }
}
