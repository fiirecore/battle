use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use pokedex::pokemon::{owned::SavedPokemon, party::Party};

use crate::{
    endpoint::{BattleEndpoint, ReceiveError},
    message::{ClientMessage, ServerMessage},
    party::{PartyIndex, PlayerParty, RemoteParty},
    pokemon::PokemonView,
    BattleData,
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

pub struct Player<ID, A: PartyIndex, P, const AS: usize> {
    pub party: PlayerParty<ID, A, P, AS>,
    endpoint: Box<dyn BattleEndpoint<ID, AS>>,
    pub settings: PlayerSettings,
}

impl<ID, A: PartyIndex, P: PokemonView, const AS: usize> Player<ID, A, P, AS> {
    pub fn new(
        id: ID,
        name: Option<String>,
        pokemon: Party<P>,
        settings: PlayerSettings,
        endpoint: Box<dyn BattleEndpoint<ID, AS>>,
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

    pub fn receive(&mut self) -> Result<ClientMessage<ID>, Option<ReceiveError>> {
        self.endpoint.receive()
    }
}

pub struct PlayerWithEndpoint<ID, const AS: usize>(
    pub LocalPlayer<ID, AS>,
    pub Box<dyn BattleEndpoint<ID, AS>>,
);

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalPlayer<ID, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    pub party: Party<SavedPokemon>,
    pub settings: PlayerSettings,
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
