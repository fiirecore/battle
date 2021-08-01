use serde::{Deserialize, Serialize};

use pokedex::item::ItemId;

use crate::BoundAction;

mod target;
pub use target::*;

#[cfg(feature = "host")]
mod queue;
#[cfg(feature = "host")]
pub use queue::*;

pub mod client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleMove {
    Move(usize, MoveTargetInstance),
    UseItem(ItemId, usize),
    Switch(usize),
}

pub type BoundBattleMove<ID> = BoundAction<ID, BattleMove>;
