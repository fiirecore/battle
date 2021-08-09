use serde::{Deserialize, Serialize};

use pokedex::pokemon::PokemonId;

use crate::{BattleData, player::{RemotePlayerKind, BattlePlayer}, party::PartyIndex};

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatedPlayer<ID, P> {
    pub id: ID,
    pub name: Option<String>,
    pub active: Vec<Option<usize>>,
    pub data: BattleData,
    pub remote: RemotePlayerKind<ID, P>,
}

impl<ID: Copy> ValidatedPlayer<ID, PokemonId> {

    pub fn new(data: BattleData, player: &BattlePlayer<ID>, other: &BattlePlayer<ID>) -> Self {
        Self {
            id: player.party.id,
            name: player.name.clone(),
            active: player.party.active.iter().map(|a| a.as_ref().map(PartyIndex::index)).collect(),
            data,
            remote: other.as_remote(),
        }
    }

}