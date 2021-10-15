use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use pokedex::pokemon::{party::Party, owned::SavedPokemon};

use crate::{
    message::{ClientMessage, ServerMessage},
    party::{PartyIndex, PlayerParty, RemoteParty},
    pokemon::PokemonView,
    BattleData, BattleEndpoint,
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

#[derive(Debug)]
pub struct Player<ID, A: PartyIndex, P, E: BattleEndpoint<ID, AS>, const AS: usize> {
    pub party: PlayerParty<ID, A, P, AS>,
    endpoint: E,
    pub settings: PlayerSettings,
}

impl<ID, A: PartyIndex, P: PokemonView, E: BattleEndpoint<ID, AS>, const AS: usize>
    Player<ID, A, P, E, AS>
{
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

    pub fn send(&mut self, message: ServerMessage<ID, AS>) {
        self.endpoint.send(message)
    }

    pub fn receive(&mut self) -> Option<ClientMessage<ID>> {
        self.endpoint.receive()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalPlayer<ID, E: BattleEndpoint<ID, AS>, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<SavedPokemon>,
    pub settings: PlayerSettings,
    pub endpoint: E,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatedPlayer<ID, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    #[serde(with = "BigArray")]
    pub active: [Option<usize>; AS],
    pub data: BattleData,
    pub remotes: Vec<RemoteParty<ID, AS>>,
}
