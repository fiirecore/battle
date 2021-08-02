use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{moves::MoveId, pokemon::PokemonInstance};

use crate::{
    moves::client::BoundClientMove,
    moves::BattleMove,
    player::ValidatedPlayer,
    pokemon::{PokemonIndex, UnknownPokemon},
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
    Validate(ValidatedPlayer<ID>),
    StartSelecting,
    Catch(PokemonInstance),
    TurnQueue(Vec<BoundClientMove<ID>>),
    ConfirmFaintReplace(ActiveIndex, bool),
    FaintReplace(PokemonIndex<ID>, usize),
    AddUnknown(PartyIndex, UnknownPokemon),
    Winner(ID),
}
