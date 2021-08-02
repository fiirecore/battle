use serde::{Deserialize, Serialize};

use crate::{BattleData, player::{RemotePlayer, BattlePlayer}, party::PartyIndex};

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatedPlayer<ID> {
    pub id: ID,
    pub name: Option<String>,
    pub active: Vec<Option<usize>>,
    pub data: BattleData,
    pub remote: RemotePlayer<ID>,
}

impl<ID: Copy> ValidatedPlayer<ID> {

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