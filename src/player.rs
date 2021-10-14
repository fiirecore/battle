use core::cell::Ref;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use pokedex::pokemon::{owned::OwnedPokemon, party::Party};

use crate::{
    message::{ClientMessage, ServerMessage},
    party::{BattleParty, RemoteParty},
    pokemon::ActivePokemon,
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
pub struct BattlePlayer<'d, ID, E: BattleEndpoint<ID, AS>, const AS: usize> {
    pub(crate) party: BattleParty<'d, ID, AS>,
    endpoint: E,
    pub settings: PlayerSettings,
}

impl<'d, ID, E: BattleEndpoint<ID, AS>, const AS: usize> BattlePlayer<'d, ID, E, AS> {
    pub fn new(
        id: ID,
        name: Option<String>,
        pokemon: Party<OwnedPokemon<'d>>,
        settings: PlayerSettings,
        endpoint: E,
    ) -> Self {
        Self {
            endpoint,
            party: BattleParty::new(id, name, pokemon.into_iter().map(Into::into).collect()),
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

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatedPlayer<ID, const AS: usize> {
    pub id: ID,
    pub name: Option<String>,
    #[serde(with = "BigArray")]
    pub active: [Option<usize>; AS],
    pub data: BattleData,
    pub remotes: Vec<RemoteParty<ID, AS>>,
}

impl<ID: Clone, const AS: usize> ValidatedPlayer<ID, AS> {
    pub fn new<
        'd: 'a,
        'a,
        E: BattleEndpoint<ID, AS>,
        I: Iterator<Item = Ref<'a, BattlePlayer<'d, ID, E, AS>>> + 'a,
    >(
        data: BattleData,
        player: &BattlePlayer<ID, E, AS>,
        others: I,
    ) -> Self
    where
        ID: 'a,
        E: 'a,
    {
        Self {
            id: player.party.id().clone(),
            name: player.party.name.clone(),
            active: ActivePokemon::into_remote(&player.party.active),
            data,
            remotes: others.map(|player| player.party.as_remote()).collect(),
        }
    }
}
