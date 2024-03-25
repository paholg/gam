use bevy_ecs::{component::Component, system::Query};
use bevy_reflect::Reflect;

use crate::{status_effect::TimeDilation, time::Dur};

#[derive(Component, Reflect, Default)]
pub struct Cooldown {
    cd: Dur,
}

impl Cooldown {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_available(&self, time_dilation: &TimeDilation) -> bool {
        self.cd.is_done(time_dilation)
    }

    pub fn set(&mut self, cooldown: Dur) {
        self.cd = cooldown;
    }

    pub fn reset(&mut self) {
        self.cd = Dur::new(0);
    }

    pub fn tick(&mut self, time_dilation: &TimeDilation) {
        self.cd.tick(time_dilation);
    }
}

pub fn global_cooldown_system(mut query: Query<(&mut Cooldown, &TimeDilation)>) {
    for (mut cd, time_dilation) in &mut query {
        cd.tick(time_dilation);
    }
}
