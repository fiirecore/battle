use serde::{Deserialize, Serialize};

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

pub struct Player<ID, A: ActivePokemon, P, E> {
    pub party: PlayerParty<ID, A, P>,
    pub settings: PlayerSettings,
    pub endpoint: E,
}

impl<ID, A: ActivePokemon, P: PokemonView, E> Player<ID, A, P, E> {
    pub fn new(
        id: ID,
        name: Option<String>,
        active: usize,
        pokemon: Party<P>,
        settings: PlayerSettings,
        endpoint: E,
    ) -> Self {
        Self {
            party: PlayerParty::new(id, name, active, pokemon),
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
pub struct ClientPlayerData<ID> {
    pub id: ID,
    pub name: Option<String>,
    pub active: Active<usize>,
    pub data: BattleData,
    pub remotes: Vec<RemoteParty<ID>>,
}
