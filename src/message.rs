use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{item::bag::SavedBag, moves::MoveId, pokemon::owned::SavedPokemon, Money};

use crate::{
    data::BattleData,
    party::{PlayerParty, RemoteParty},
    player::RemovalReason,
    pokemon::{ActivePosition, Indexed, PartyPosition, PokemonView},
    select::{BattleSelection, ClientAction, SelectMessage},
};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum ClientMessage<ID> {
    Select(ActivePosition, BattleSelection<ID>),
    TryForfeit,
    LearnMove(PartyPosition, MoveId, Option<usize>), // pokemon index, move, move index
                                                     // RequestMoveData(MoveId),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ServerMessage<ID, T> {
    /// Request the client to select from an active position
    Select(ActivePosition, SelectMessage),

    /// Send the turns to a player
    Results(Vec<ClientAction<ID>>),

    /// Replace fainted pokemon
    Replace(Indexed<ID, usize>),
    Reveal(Indexed<ID, PokemonView>),

    PlayerData(
        BattleData,
        PlayerParty<ID, usize, SavedPokemon, T>,
        SavedBag,
    ),
    AddOpponent(RemoteParty<ID, T>),

    Remove(ID, RemovalReason, Money),
    End(Option<ID>),
    // MoveData(BattleMove),
}

// #[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
// pub enum TimedAction {
//     Selecting,
//     Replace,
// }

// #[derive(Debug, Clone, Deserialize, Serialize)]
// pub enum FailedAction {
//     Move(ActivePosition),
//     Switch(ActivePosition),
//     Replace(ActivePosition),
// }
