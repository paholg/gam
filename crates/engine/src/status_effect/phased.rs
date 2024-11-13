use bevy_ecs::component::Component;
use bevy_ecs::system::Query;

use super::TimeDilation;
use crate::time::Dur;

/// Phased is a boolean condition.
///
/// Phased things interact with only other phased things. So, for example, a
/// phased character can move through walls, is invulnerable to normal damage/
/// effects, but cannot hurt anyone. However, a phased enemy could fight them
/// like normal.
#[derive(Component, Debug, Default)]
pub struct Phased {
    val: bool,
    duration: Dur,
}

impl Phased {
    fn toggle(&mut self) {
        self.val = !self.val
    }

    fn tick(&mut self, time_dilation: &TimeDilation) {
        if !self.duration.is_done(time_dilation) {
            self.duration.tick(time_dilation);
            if self.duration.is_done(time_dilation) {
                self.toggle();
            }
        }
    }
}

pub fn phased_tick(mut query: Query<(&mut Phased, &TimeDilation)>) {
    for (mut phased, dilation) in &mut query {
        phased.tick(dilation);
    }
}
