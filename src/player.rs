use serde::{Deserialize, Serialize};

use pokedex::{
    item::bag::{Bag, SavedBag},
    pokemon::{owned::SavedPokemon, party::Party},
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

pub struct Player<ID, A: ActivePokemon, P, I, E> {
    pub party: PlayerParty<ID, A, P>,
    pub bag: Bag<I>,
    pub settings: PlayerSettings,
    pub endpoint: E,
}

impl<ID, A: ActivePokemon, P: PokemonView, I, E> Player<ID, A, P, I, E> {
    pub fn new(
        id: ID,
        name: Option<String>,
        active: usize,
        pokemon: Party<P>,
        bag: Bag<I>,
        settings: PlayerSettings,
        endpoint: E,
    ) -> Self {
        Self {
            party: PlayerParty::new(id, name, active, pokemon),
            bag,
            settings,
            endpoint,
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
    pub data: BattleData,
    pub local: PlayerParty<ID, usize, SavedPokemon>,
    pub remotes: Vec<RemoteParty<ID>>,
    pub bag: SavedBag,
}
