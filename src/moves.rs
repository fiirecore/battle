use serde::{Deserialize, Serialize};

use pokedex::item::ItemId;

use crate::BoundAction;

pub mod usage;

#[cfg(feature = "host")]
mod queue;
#[cfg(feature = "host")]
pub use queue::*;

pub mod client;

pub mod persistent;

pub type Move = pokedex::moves::Move<usage::MoveUsage>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BattleMove {
    Move(usize, usage::MoveTargetInstance),
    UseItem(ItemId, usize),
    Switch(usize),
}

pub type BoundBattleMove<ID> = BoundAction<ID, BattleMove>;
