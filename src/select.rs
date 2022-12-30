use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    item::ItemId,
    moves::{MoveId, PP},
    pokemon::{Experience},
};

use crate::{
    moves::{ClientDamage, MoveCancelId, RemovePokemonId},
    pokemon::{
        stat::{BattleStatType, Stage},
        Indexed, PartyPosition, TeamIndex,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleSelection<ID> {
    /// Move ID, and its optional target.
    Move(MoveId, Option<TeamIndex<ID>>),
    /// Switch to another pokemon with it's party index.
    /// OR Replace a fainted pokemon
    Pokemon(PartyPosition),
    Item(Indexed<ID, ItemId>),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SelectMessage {
    Request(Option<SelectReason>),
    Confirm(SelectConfirm),
    Deny,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SelectConfirm {
    Move(MoveId, PP),
    Other,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SelectReason {
    InvalidInput,
    NoPP,
    MissingAction,
    MissingPokemon,
    MissingActive,
    FaintedPokemon,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientAction<ID> {
    /// turn number, turn user, turn type
    Announce(usize, Option<TeamIndex<ID>>, ClientActionType<ID>),
    /// vector of targets of client move action
    Actions(Vec<Indexed<ID, PublicAction>>),
    Error(#[serde(skip)] String),
}

/// Precedes move actions in the queue
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientActionType<ID> {
    Move(MoveId),
    Pokemon(PartyPosition),
    Item(Indexed<ID, ItemId>),
    Post,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PublicAction {
    /// This contains the percent HP the pokemon was left at, how effective the attack was, and if it was a critical hit.
    /// A Pokemon faints when it's hp is set to 0.0
    SetHP(ClientDamage<f32>),
    AddStat(BattleStatType, Stage),
    Ailment(Option<LiveAilment>),
    Switch(PartyPosition),
    Reveal,
    Cancel(MoveCancelId),
    Remove(RemovePokemonId),
    Miss,
}

pub enum PrivateAction {
    AddExp(Experience),
}

impl<ID: core::fmt::Display> core::fmt::Display for BattleSelection<ID> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Move(index, ..) => write!(f, "Move {}", &index.0),
            Self::Item(Indexed(.., id)) => write!(f, "Item {}", id.as_str()),
            Self::Pokemon(index) => write!(f, "Switch to {}", index),
        }
    }
}
