use bevy::prelude::Color;
use itertools::Itertools;
use smallvec::SmallVec;

pub struct ColorGradient {
    pub colors: SmallVec<(f32, Color), 4>,
}

impl ColorGradient {
    /// Construct a new color gradient
    ///

    pub fn new(colors: impl IntoIterator<Item = (f32, Color)>) -> Self {
        let colors = colors.into_iter().collect();
        let this = Self { colors };

        #[cfg(debug_assertions)]
        {
            assert_eq!(this.colors[0].0, 0.0);
            assert_eq!(this.colors.last().unwrap().0, 1.0);

            let mut val = 0.0;
            for &(next, _) in &this.colors[1..] {
                assert!(next > val, "Expected {next} > {val}");
                val = next;
            }
        }
        this
    }

    pub fn get(&self, val: f32) -> Color {
        for (&(low, color_low), &(high, color_high)) in self.colors.iter().tuple_windows() {
            if val >= low && val <= high {
                let low_frac = (high - val) / (high - low);
                let high_frac = (val - low) / (high - low);

                return color_low * low_frac + color_high * high_frac;
            }
        }

        self.colors.last().unwrap().1
    }
}
