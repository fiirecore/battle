use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use pokedex::{
    item::bag::{InitBag, SavedBag},
    pokemon::owned::SavedPokemon,
};

use crate::{
    data::BattleData,
    party::{ActivePokemon, PlayerParty, RemoteParty},
    pokemon::PokemonView,
};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct PlayerSettings {
    pub gains_exp: bool,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self { gains_exp: true }
    }
}

pub struct Player<ID, A: ActivePokemon, P, T, E> {
    pub party: PlayerParty<ID, A, P, T>,
    pub bag: InitBag,
    pub settings: PlayerSettings,
    pub endpoint: E,
}

impl<ID, A: ActivePokemon, P: PokemonView, T, E> Player<ID, A, P, T, E> {
    pub fn id(&self) -> &ID {
        self.party.id()
    }

    pub fn name(&self) -> &str {
        self.party.name()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientPlayerData<ID, T> {
    pub data: BattleData,
    pub local: PlayerParty<ID, usize, SavedPokemon, T>,
    pub remotes: Vec<RemoteParty<ID, T>>,
    pub bag: SavedBag,
}
