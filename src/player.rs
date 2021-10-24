use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use pokedex::pokemon::party::Party;

use crate::{
    data::BattleData,
    party::{Active, ActivePokemon, PlayerParty, RemoteParty},
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

pub struct Player<ID, A: ActivePokemon, P, E, const AS: usize> {
    pub party: PlayerParty<ID, A, P, AS>,
    pub settings: PlayerSettings,
    pub endpoint: E,
}

impl<ID, A: ActivePokemon, P: PokemonView, E, const AS: usize> Player<ID, A, P, E, AS> {
    pub fn new(
        id: ID,
        name: Option<String>,
        pokemon: Party<P>,
        settings: PlayerSettings,
        endpoint: E,
    ) -> Self {
        Self {
            party: PlayerParty::new(id, name, pokemon),
            endpoint,
            settings,
        }
    }

    pub fn id(&self) -> &ID {
        self.party.id()
    }

    pub fn name(&self) -> &str {
        self.party.name()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientPlayerData<ID, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    #[serde(with = "BigArray")]
    pub active: Active<usize, AS>,
    pub data: BattleData,
    pub remotes: Vec<RemoteParty<ID, AS>>,
}
