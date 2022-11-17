use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum Ability {
    #[default]
    None,
    HyperSprint,
}
