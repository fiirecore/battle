use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use pokedex::{
    item::ItemId,
    moves::{usage::MoveResult, MoveRef},
};

use crate::BoundAction;

mod target;
pub use target::*;

mod queue;
pub use queue::*;

pub mod client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleMove {
    Move(usize, MoveTargetInstance),
    UseItem(ItemId, usize),
    Switch(usize),
}

pub type BoundBattleMove<ID> = BoundAction<ID, BattleMove>;

pub type MoveResults = BTreeMap<MoveTargetLocation, Vec<MoveResult>>;

pub struct TurnResult {
    pub pokemon_move: MoveRef,
    pub results: MoveResults,
}
