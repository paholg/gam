use crate::math::F32;

use glam::{Vec2, Vec3};

pub struct V2 {
    pub x: F32,
    pub y: F32,
}

impl From<Vec2> for V2 {
    fn from(v: Vec2) -> Self {
        Self {
            x: v.x.into(),
            y: v.y.into(),
        }
    }
}

impl V2 {
    #[inline]
    pub fn new(v: Vec2) -> Self {
        Self {
            x: F32::new(v.x),
            y: F32::new(v.y),
        }
    }
}

pub struct V3 {
    pub x: F32,
    pub y: F32,
    pub z: F32,
}

impl From<Vec3> for V3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x.into(),
            y: v.y.into(),
            z: v.z.into(),
        }
    }
}
impl V3 {
    pub const X: V3 = Vec3::X.into();

    #[inline]
    pub fn new(v: Vec3) -> Self {
        Self {
            x: F32::new(v.x),
            y: F32::new(v.y),
            z: F32::new(v.z),
        }
    }
}
