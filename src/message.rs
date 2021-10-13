use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{
    moves::MoveId,
    pokemon::{owned::SavedPokemon, PokemonId},
};

use crate::{
    moves::{BattleMove, ClientMove},
    player::ValidatedPlayer,
    pokemon::{battle::UninitUnknownPokemon, ActivePosition, PartyPosition, PokemonIndex},
    BoundAction,
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
    Catch(SavedPokemon),
    TurnQueue(Vec<BoundAction<ID, ClientMove>>),
    ConfirmFaintReplace(ActivePosition, bool),
    FaintReplace(PokemonIndex<ID>, usize),
    AddUnknown(ID, PartyPosition, UninitUnknownPokemon),
    End(EndState),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum EndState {
    Win,  // add money gained
    Lose, // add money lost
    Other,
}
