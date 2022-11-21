use std::{fmt, hash::Hasher};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
pub struct F32(pub f32);

impl fmt::Display for F32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for F32 {
    fn eq(&self, rhs: &F32) -> bool {
        if self.0.is_nan() && rhs.0.is_nan() {
            true
        } else {
            self.0 == rhs.0
        }
    }
}

impl Eq for F32 {}

impl std::hash::Hash for F32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}

impl From<F32> for f32 {
    fn from(value: F32) -> Self {
        value.0
    }
}

impl From<f32> for F32 {
    fn from(value: f32) -> Self {
        F32(value)
    }
}
