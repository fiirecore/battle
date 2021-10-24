use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    item::ItemId,
    moves::{MoveId, PP},
    pokemon::{Experience, Level},
};

use crate::pokemon::{
    stat::{BattleStatType, Stage},
    Indexed, PokemonIdentifier,
};

pub mod damage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleMove<ID> {
    /// Move (by its index), and its optional target.
    Move(usize, Option<PokemonIdentifier<ID>>),
    UseItem(Indexed<ID, ItemId>),
    Switch(usize),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMove<ID> {
    /// Id of move, PP lost from using the move, client move actions
    Move(MoveId, PP, Vec<Indexed<ID, ClientMoveAction>>),
    UseItem(Indexed<ID, ItemId>),
    Switch(usize),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ClientMoveAction {
    /// This contains the percent HP the pokemon was left at, how effective the attack was, and if it was a critical hit.
    /// A Pokemon faints when it's hp is set to 0.0
    SetHP(damage::ClientDamage<f32>),
    AddStat(BattleStatType, Stage),
    Ailment(LiveAilment),

    Flinch,
    Miss,

    SetExp(Experience, Level),

    Error,
}

pub type Critical = bool;
/// 0 through 100
pub type Percent = u8;

impl<ID: core::fmt::Display> core::fmt::Display for BattleMove<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleMove::Move(index, ..) => write!(f, "Move #{}", index),
            BattleMove::UseItem(Indexed(.., id)) => write!(f, "Item {}", id),
            BattleMove::Switch(index) => write!(f, "Switch to {}", index),
        }
    }
}
