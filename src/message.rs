use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{moves::MoveId, pokemon::{PokemonId, OwnedIdPokemon}};

use crate::{
    moves::client::BoundClientMove,
    moves::BattleMove,
    player::ValidatedPlayer,
    pokemon::{PokemonIndex, UninitUnknownPokemon},
};

type ActiveIndex = usize;
type PartyIndex = usize;

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    Move(ActiveIndex, BattleMove),         // active pokemon, move
    ReplaceFaint(ActiveIndex, PartyIndex), // active pokemon, party index
    FinishedTurnQueue,
    Forfeit,
    LearnMove(PartyIndex, MoveId, u8), // pokemon index, move, move index (0 - 3)
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage<ID> {
    Begin(ValidatedPlayer<ID, PokemonId>),
    StartSelecting,
    Catch(OwnedIdPokemon),
    TurnQueue(Vec<BoundClientMove<ID>>),
    ConfirmFaintReplace(ActiveIndex, bool),
    FaintReplace(PokemonIndex<ID>, usize),
    AddUnknown(PartyIndex, UninitUnknownPokemon),
    Winner(Option<ID>),
}
