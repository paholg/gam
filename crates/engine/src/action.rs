pub const ABILITY_COUNT: usize = 5;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub enum Action {
    Ability0 = 0,
    Ability1 = 1,
    Ability2 = 2,
    Ability3 = 3,
    Ability4 = 4,
    Move,
    Aim,
    Menu,
}
