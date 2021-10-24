use core::fmt::Debug;
use serde::{Deserialize, Serialize};

use pokedex::{moves::MoveId, pokemon::owned::SavedPokemon};

use crate::{
    moves::{BattleMove, ClientMove},
    player::ClientPlayerData,
    pokemon::{remote::RemotePokemon, ActivePosition, Indexed, PartyPosition},
};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum ClientMessage<ID> {
    Move(ActivePosition, BattleMove<ID>),
    ReplaceFaint(ActivePosition, PartyPosition),
    Forfeit,
    LearnMove(PartyPosition, MoveId, u8), // pokemon index, move, move index (0 - 3)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ServerMessage<ID, const AS: usize> {
    Begin(ClientPlayerData<ID, AS>),

    Start(StartableAction<ID>),

    Ping(TimedAction),
    Fail(FailedAction),

    AddRemote(Indexed<ID, RemotePokemon>),
    Replace(Indexed<ID, usize>),

    Catch(SavedPokemon),

    End,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StartableAction<ID> {
    Selecting,
    Turns(Vec<Indexed<ID, ClientMove<ID>>>),
}

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TimedAction {
    Selecting,
    Replace,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FailedAction {
    Move(ActivePosition),
    Switch(ActivePosition),
    Replace(ActivePosition),
}

// #[derive(Debug, Clone, Copy, Deserialize, Serialize)]
// pub enum EndState {
//     Win,  // add money gained
//     Lose, // add money lost
//     Other,
// }
