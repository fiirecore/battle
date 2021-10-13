use core::cell::Ref;

use serde::{Deserialize, Serialize};

use pokedex::pokemon::PokemonId;

use crate::{
    party::PartyIndex,
    player::{BattlePlayer, RemotePlayerKind},
    BattleData, BattleEndpoint,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatedPlayer<ID, P> {
    pub id: ID,
    pub name: Option<String>,
    pub active: Vec<Option<usize>>,
    pub data: BattleData,
    pub remotes: Vec<RemotePlayerKind<ID, P>>,
}

impl<ID: Copy> ValidatedPlayer<ID, PokemonId> {
    pub fn new<
        'd: 'a,
        'a,
        E: BattleEndpoint<ID>,
        I: Iterator<Item = Ref<'a, BattlePlayer<'d, ID, E>>> + 'a,
    >(
        data: BattleData,
        player: &BattlePlayer<ID, E>,
        others: I,
    ) -> Self
    where
        ID: 'a, E: 'a,
    {
        Self {
            id: player.party.id,
            name: player.name.clone(),
            active: player
                .party
                .active
                .iter()
                .map(|a| a.as_ref().map(PartyIndex::index))
                .collect(),
            data,
            remotes: others.map(|p| p.as_remote()).collect(),
        }
    }
}
