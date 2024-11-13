use core::fmt;

use bevy_ecs::system::Resource;
use bevy_math::Vec2;
use bevy_reflect::TypePath;
use bevy_utils::HashMap;
use bitmask_enum::bitmask;
use bytemuck::Pod;
use bytemuck::Zeroable;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumIter;

use crate::ability::Abilities;
use crate::ability::Ability;
use crate::Player;

/// The inputs of all players
#[derive(Resource, Default, Debug)]
pub struct PlayerInputs {
    map: HashMap<Player, Input>,
}

impl PlayerInputs {
    pub fn get(&self, player: &Player) -> Option<&Input> {
        self.map.get(player)
    }

    pub fn insert(&mut self, player: Player, input: Input) {
        self.map.insert(player, input);
    }
}

#[derive(EnumIter, TypePath, Deserialize)]
#[bitmask(u16)]
pub enum Action {
    Ability0,
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Menu,
}

impl Action {
    pub fn abilities_fired(&self, abilities: &Abilities) -> impl Iterator<Item = Ability> {
        let mut fired = Vec::new();
        if self.contains(Action::Ability0) {
            fired.push(abilities[0]);
        }
        if self.contains(Action::Ability1) {
            fired.push(abilities[1]);
        }
        if self.contains(Action::Ability2) {
            fired.push(abilities[2]);
        }
        if self.contains(Action::Ability3) {
            fired.push(abilities[3]);
        }
        if self.contains(Action::Ability4) {
            fired.push(abilities[4]);
        }

        fired.into_iter()
    }
}

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl Default for Action {
    fn default() -> Self {
        Self::none()
    }
}

/// A single byte that can be converted to/from a f32 in the range [-1.0, 1.0].
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default)]
#[repr(transparent)]
pub struct BoundedF8(i8);

impl fmt::Debug for BoundedF8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&f32::from(*self), f)
    }
}

impl From<BoundedF8> for f32 {
    fn from(value: BoundedF8) -> Self {
        if value.0.is_negative() {
            ((value.0 + 1) as f32) / 127.0
        } else {
            (value.0 as f32) / 127.0
        }
    }
}

impl From<f32> for BoundedF8 {
    fn from(value: f32) -> Self {
        let value = value.clamp(-1.0, 1.0) * 127.0;

        let byte = if value.is_sign_negative() {
            (value - 1.0) as i8
        } else {
            value as i8
        };
        Self(byte)
    }
}

/// A user input, as sent over the network.
#[derive(Copy, Clone, PartialEq, Pod, Zeroable, Default, Debug)]
#[repr(C)]
pub struct Input {
    buttons: u16,
    move_x: BoundedF8,
    move_z: BoundedF8,
    cursor_x: f32,
    cursor_z: f32,
}

impl Input {
    pub fn new(buttons: Action, movement: Vec2, cursor: Vec2) -> Self {
        Self {
            buttons: buttons.bits(),
            move_x: movement.x.into(),
            move_z: movement.y.into(),
            cursor_x: cursor.x,
            cursor_z: cursor.y,
        }
    }

    pub fn buttons(&self) -> Action {
        Action::from(self.buttons)
    }

    pub fn movement(&self) -> Vec2 {
        Vec2::new(self.move_x.into(), self.move_z.into())
    }

    pub fn cursor(&self) -> Option<Vec2> {
        if self.cursor_x.is_finite() && self.cursor_z.is_finite() {
            Some(Vec2::new(self.cursor_x, self.cursor_z))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::BoundedF8;

    #[test]
    fn movement_conversion() {
        for (byte, expected) in [(0, 0.0), (i8::MIN, -1.0), (i8::MAX, 1.0)] {
            let bounded = BoundedF8(byte);
            assert_eq!(f32::from(bounded), expected);
            assert_eq!(BoundedF8::from(expected), bounded);
        }
    }
}
