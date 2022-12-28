use serde::{Deserialize, Serialize};

use pokedex::{
    ailment::LiveAilment,
    item::ItemId,
    moves::{MoveId, PP},
    pokemon::{Experience, Level},
};

use crate::{
    engine::{ActionResult, BattlePokemon},
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
    Pokemon(usize),
    Item(Indexed<ID, ItemId>),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SelectMessage {
    Request(Option<SelectReason>),
    Confirm(SelectConfirm),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SelectConfirm {
    Move(MoveId, PP),
    Other,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum SelectReason {
    NoPP,
    MissingAction,
    MissingPokemon,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientAction<ID> {
    /// turn number, turn user, turn type
    Announce(usize, Option<TeamIndex<ID>>, ClientActionType<ID>),
    Actions(Vec<Indexed<ID, ClientMoveAction>>),
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
pub enum ClientMoveAction {
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

    SetExp(Experience, Level),

    Error,
}

impl ClientMoveAction {
    pub fn new(pokemon: &BattlePokemon, result: ActionResult) -> Self {
        match result {
            ActionResult::Damage(_) => todo!(),
            ActionResult::Heal(_) => todo!(),
            ActionResult::Ailment(_) => todo!(),
            ActionResult::Stat(_, _) => todo!(),
            ActionResult::Reveal(_) => todo!(),
            ActionResult::Cancel(_) => todo!(),
            ActionResult::Remove(_) => todo!(),
            ActionResult::Fail => todo!(),
            ActionResult::Miss => todo!(),
        }
    }
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
