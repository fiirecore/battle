use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{
    moves::MoveId,
    pokemon::{PokemonInstance, PokemonParty},
};

use crate::{
    moves::client::BoundClientMove,
    moves::BattleMove,
    pokemon::{PokemonIndex, UnknownPokemon},
    player::{RemotePlayer, LocalPlayer},
    BattleData,
};

type ActiveIndex = usize;
type PartyIndex = usize;

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
    // Connect(BattleParty),
    Move(ActiveIndex, BattleMove), // active pokemon, move
    FaintReplace(ActiveIndex, PartyIndex), // active pokemon, party index
    RequestPokemon(PartyIndex),
    FinishedTurnQueue,
    AddLearnedMove(PartyIndex, usize, MoveId), // pokemon index, move index, move
    Forfeit,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage<ID> {
    User(BattleData, LocalPlayer<ID>),
    Opponents(RemotePlayer<ID>),
    // UpdatePokemon(TrainerId, usize, UnknownPokemon),
    PokemonRequest(usize, PokemonInstance),
    StartSelecting,
    TurnQueue(Vec<BoundClientMove<ID>>),
    // AskFinishedTurnQueue,
    // SelectMoveError(usize),
    // Catch(PokemonIndex),
    // RequestFaintReplace(Active),
    CanFaintReplace(usize, bool),
    FaintReplace(PokemonIndex<ID>, Option<usize>),
    AddUnknown(PartyIndex, UnknownPokemon),
    Winner(ID), // party is for when user requests party back. used in remote clients
    PartyRequest(PokemonParty),
}
