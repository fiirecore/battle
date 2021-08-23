use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{
    moves::MoveId,
    pokemon::{OwnedIdPokemon, PokemonId},
};

use crate::{
    BoundAction,
    moves::{BattleMove, ClientMove},
    player::ValidatedPlayer,
    pokemon::{battle::UninitUnknownPokemon, ActivePosition, PartyPosition, PokemonIndex},
};

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    Move(ActivePosition, BattleMove),            // active pokemon, move
    ReplaceFaint(ActivePosition, PartyPosition), // active pokemon, party index
    FinishedTurnQueue,
    Forfeit,
    LearnMove(PartyPosition, MoveId, u8), // pokemon index, move, move index (0 - 3)
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage<ID> {
    Begin(ValidatedPlayer<ID, PokemonId>),
    StartSelecting,
    Catch(OwnedIdPokemon),
    TurnQueue(Vec<BoundAction<ID, ClientMove>>),
    ConfirmFaintReplace(ActivePosition, bool),
    FaintReplace(PokemonIndex<ID>, usize),
    AddUnknown(PartyPosition, UninitUnknownPokemon),
    Winner(Option<ID>),
}
